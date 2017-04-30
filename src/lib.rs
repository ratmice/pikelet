extern crate cgmath;
#[cfg(test)]
extern crate expectest;
extern crate find_folder;
extern crate fnv;
extern crate glium;
#[macro_use]
extern crate imgui;
extern crate notify;
extern crate num_traits;
extern crate rand;

extern crate dggs;
extern crate engine;
extern crate geom;
extern crate geomath;
extern crate job_queue;

use cgmath::prelude::*;
use cgmath::{Matrix4, PerspectiveFov, Point2, Point3, Rad, Vector3};
use find_folder::Search as FolderSearch;
use glium::glutin;
use std::sync::mpsc::Sender;

use engine::{Application, FrameMetrics, Loop, RenderData};
use engine::camera::{Camera, ComputedCamera};
use engine::color;
use engine::math::Size2;
use engine::render::{CommandList, ResourceEvent};

use self::debug_controls::DebugControls;
use self::job::Job;

mod job;
mod debug_controls;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    Close,
    SetLimitingFps(bool),
    SetPlanetRadius(f32),
    SetPlanetSubdivisions(usize),
    SetShowingStarField(bool),
    SetStarFieldRadius(f32),
    SetUiCapturingMouse(bool),
    SetWireframe(bool),
    ToggleUi,
    ResetState,
    DragStart,
    DragEnd,
    ZoomStart,
    ZoomEnd,
    MousePosition(Point2<i32>),
    NoOp,
}

impl From<glutin::Event> for InputEvent {
    fn from(src: glutin::Event) -> InputEvent {
        use glium::glutin::ElementState::*;
        use glium::glutin::Event;
        use glium::glutin::MouseButton;
        use glium::glutin::VirtualKeyCode as Key;

        match src {
            Event::Closed |
            Event::KeyboardInput(Pressed, _, Some(Key::Escape)) => InputEvent::Close,
            Event::KeyboardInput(Pressed, _, Some(Key::R)) => InputEvent::ResetState,
            Event::KeyboardInput(Pressed, _, Some(Key::U)) => InputEvent::ToggleUi,
            Event::MouseInput(Pressed, MouseButton::Left) => InputEvent::DragStart,
            Event::MouseInput(Released, MouseButton::Left) => InputEvent::DragEnd,
            Event::MouseInput(Pressed, MouseButton::Right) => InputEvent::ZoomStart,
            Event::MouseInput(Released, MouseButton::Right) => InputEvent::ZoomEnd,
            Event::MouseMoved(x, y) => InputEvent::MousePosition(Point2::new(x, y)),
            _ => InputEvent::NoOp,
        }
    }
}

#[derive(Clone, Debug)]
struct StarField {
    count: usize,
    size: f32,
}

#[derive(Clone)]
struct State {
    is_dragging: bool,
    is_limiting_fps: bool,
    is_showing_star_field: bool,
    is_ui_enabled: bool,
    is_ui_capturing_mouse: bool,
    is_wireframe: bool,
    is_zooming: bool,

    frame_metrics: FrameMetrics,
    mouse_position: Point2<i32>,

    light_dir: Vector3<f32>,

    camera_rotation: Rad<f32>,
    camera_rotation_delta: Rad<f32>,
    camera_xz_radius: f32,
    camera_y_height: f32,
    camera_near: f32,
    camera_far: f32,
    camera_zoom_factor: f32,
    camera_drag_factor: f32,

    star_field_radius: f32,
    stars0: StarField,
    stars1: StarField,
    stars2: StarField,

    planet_radius: f32,
    planet_subdivs: usize,
}

impl State {
    fn new(frame_metrics: FrameMetrics) -> State {
        State {
            is_dragging: false,
            is_limiting_fps: true,
            is_showing_star_field: true,
            is_ui_enabled: true,
            is_ui_capturing_mouse: false,
            is_wireframe: false,
            is_zooming: false,

            light_dir: Vector3::new(0.0, 1.0, 0.2),

            frame_metrics: frame_metrics,
            mouse_position: Point2::origin(),

            camera_rotation: Rad(0.0),
            camera_rotation_delta: Rad(0.0),
            camera_xz_radius: 2.0,
            camera_y_height: 1.0,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_zoom_factor: 10.0,
            camera_drag_factor: 10.0,

            star_field_radius: 10.0,
            stars0: StarField {
                size: 1.0,
                count: 100000,
            },
            stars1: StarField {
                size: 2.5,
                count: 10000,
            },
            stars2: StarField {
                size: 5.0,
                count: 1000,
            },

            planet_radius: 1.0,
            planet_subdivs: 3,
        }
    }

    fn reset(&mut self) {
        *self = State::new(self.frame_metrics);
    }

    fn scene_camera_position(&self) -> Point3<f32> {
        Point3 {
            x: Rad::sin(self.camera_rotation) * self.camera_xz_radius,
            y: self.camera_y_height,
            z: Rad::cos(self.camera_rotation) * self.camera_xz_radius,
        }
    }

    fn scene_camera_projection(&self) -> PerspectiveFov<f32> {
        PerspectiveFov {
            aspect: self.frame_metrics.aspect_ratio(),
            fovy: Rad::full_turn() / 6.0,
            near: self.camera_near,
            far: self.camera_far,
        }
    }

    fn create_scene_camera(&self) -> ComputedCamera {
        Camera {
                position: self.scene_camera_position(),
                target: Point3::origin(),
                projection: self.scene_camera_projection(),
            }
            .compute()
    }

    fn create_hud_camera(&self) -> Matrix4<f32> {
        let Size2 { width, height } = self.frame_metrics.size_pixels;
        cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
    }
}

pub struct Game {
    state: State,
    job_tx: Sender<Job>,
}

impl Application for Game {
    type Event = InputEvent;

    fn init(frame_metrics: FrameMetrics, resource_tx: Sender<ResourceEvent>) -> Game {
        use job_queue;
        use std::fs::File;
        use std::io;
        use std::io::prelude::*;
        use std::path::Path;

        let game = Game {
            state: State::new(frame_metrics),
            job_tx: {
                let resource_tx = resource_tx.clone();
                job_queue::spawn(move |job| Job::process(job, &resource_tx))
            },
        };

        game.queue_regenete_stars_jobs();
        game.queue_regenete_planet_job();

        let assets = FolderSearch::ParentsThenKids(3, 3)
            .for_folder("resources")
            .expect("Could not locate `resources` folder");

        fn load_shader(path: &Path) -> io::Result<String> {
            let mut file = File::open(path)?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            Ok(buffer)
        }

        resource_tx
            .send((ResourceEvent::CompileProgram {
                       name: "flat_shaded".to_string(),
                       vertex_shader: load_shader(&assets.join("shaders/flat_shaded.v.glsl"))
                           .unwrap(),
                       fragment_shader: load_shader(&assets.join("shaders/flat_shaded.f.glsl"))
                           .unwrap(),
                   }))
            .unwrap();

        resource_tx
            .send((ResourceEvent::CompileProgram {
                       name: "text".to_string(),
                       vertex_shader: load_shader(&assets.join("shaders/text.v.glsl")).unwrap(),
                       fragment_shader: load_shader(&assets.join("shaders/text.f.glsl")).unwrap(),
                   }))
            .unwrap();

        resource_tx
            .send((ResourceEvent::CompileProgram {
                       name: "unshaded".to_string(),
                       vertex_shader: load_shader(&assets.join("shaders/unshaded.v.glsl")).unwrap(),
                       fragment_shader: load_shader(&assets.join("shaders/unshaded.f.glsl"))
                           .unwrap(),
                   }))
            .unwrap();

        fn load_font(path: &Path) -> io::Result<Vec<u8>> {
            let mut file = File::open(path)?;
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;

            Ok(buffer)
        }

        resource_tx
            .send((ResourceEvent::UploadFont {
                       name: "blogger_sans".to_string(),
                       data: load_font(&assets.join("fonts/blogger_sans.ttf")).unwrap(),
                   }))
            .unwrap();

        game
    }

    fn handle_frame_request(&mut self, frame_metrics: FrameMetrics) -> Loop {
        self.state.frame_metrics = frame_metrics;
        self.state.camera_rotation -= self.state.camera_rotation_delta;

        if self.state.is_dragging {
            self.state.camera_rotation_delta = Rad(0.0);
        }

        Loop::Continue
    }

    fn handle_input(&mut self, event: InputEvent) -> Loop {
        use self::InputEvent::*;

        match event {
            Close => return Loop::Break,
            SetLimitingFps(value) => self.state.is_limiting_fps = value,
            SetPlanetRadius(value) => self.state.planet_radius = value,
            SetPlanetSubdivisions(planet_subdivs) => {
                self.state.planet_subdivs = planet_subdivs;
                self.queue_regenete_planet_job();
            },
            SetShowingStarField(value) => self.state.is_showing_star_field = value,
            SetStarFieldRadius(value) => self.state.star_field_radius = value,
            SetUiCapturingMouse(value) => self.state.is_ui_capturing_mouse = value,
            SetWireframe(value) => self.state.is_wireframe = value,
            ToggleUi => self.state.is_ui_enabled = !self.state.is_ui_enabled,
            ResetState => self.state.reset(),
            DragStart => {
                if !self.state.is_ui_capturing_mouse {
                    self.state.is_dragging = true
                }
            },
            DragEnd => self.state.is_dragging = false,
            ZoomStart => self.state.is_zooming = true,
            ZoomEnd => self.state.is_zooming = false,
            MousePosition(position) => self.handle_mouse_update(position),
            NoOp => {},
        }

        Loop::Continue
    }

    fn render(&self) -> RenderData<InputEvent> {
        RenderData {
            metrics: self.state.frame_metrics,
            is_limiting_fps: self.state.is_limiting_fps,
            command_list: self.create_command_list(),
        }
    }
}

impl Game {
    fn queue_job(&self, job: Job) {
        self.job_tx.send(job).expect("Failed queue job");
    }

    fn queue_regenete_planet_job(&self) {
        self.queue_job(Job::Planet { subdivs: self.state.planet_subdivs });
    }

    fn queue_regenete_stars_jobs(&self) {
        self.queue_job(Job::Stars {
                           index: 2,
                           count: self.state.stars2.count,
                       });
        self.queue_job(Job::Stars {
                           index: 1,
                           count: self.state.stars1.count,
                       });
        self.queue_job(Job::Stars {
                           index: 0,
                           count: self.state.stars0.count,
                       });
    }

    fn handle_mouse_update(&mut self, new_position: Point2<i32>) {
        use std::mem;

        let old_position = mem::replace(&mut self.state.mouse_position, new_position);
        let mouse_delta = new_position - old_position;

        if !self.state.is_ui_capturing_mouse {
            if self.state.is_dragging {
                let size_points = self.state.frame_metrics.size_points;
                let rotations_per_second = (mouse_delta.x as f32 / size_points.width as f32) *
                                           self.state.camera_drag_factor;
                self.state.camera_rotation_delta = Rad::full_turn() * rotations_per_second *
                                                   self.state.frame_metrics.delta_time;
            }

            if self.state.is_zooming {
                let zoom_delta = mouse_delta.x as f32 * self.state.frame_metrics.delta_time;
                self.state.camera_xz_radius -= zoom_delta * self.state.camera_zoom_factor;
            }
        }
    }

    fn create_command_list(&self) -> CommandList<InputEvent> {
        let mut command_list = CommandList::new();

        let camera = self.state.create_scene_camera();
        let screen_matrix = self.state.create_hud_camera();

        command_list.clear(color::BLUE);

        if self.state.is_showing_star_field {
            let star_field_matrix = Matrix4::from_translation(camera.position.to_vec()) *
                                    Matrix4::from_scale(self.state.star_field_radius);

            command_list.points("stars0",
                                self.state.stars0.size,
                                color::WHITE,
                                star_field_matrix,
                                camera);
            command_list.points("stars1",
                                self.state.stars1.size,
                                color::WHITE,
                                star_field_matrix,
                                camera);
            command_list.points("stars2",
                                self.state.stars2.size,
                                color::WHITE,
                                star_field_matrix,
                                camera);
        }

        let planet_matrix = Matrix4::from_scale(self.state.planet_radius);

        if self.state.is_wireframe {
            command_list.lines("planet", 0.5, color::BLACK, planet_matrix, camera);
        } else {
            command_list.solid("planet",
                               self.state.light_dir,
                               color::GREEN,
                               planet_matrix,
                               camera);
        }

        if self.state.is_ui_enabled {
            let size_points = self.state.frame_metrics.size_points;
            let (scale_x, scale_y) = self.state.frame_metrics.framebuffer_scale();

            let text = format!("{:.2}", self.state.frame_metrics.frames_per_second());
            let font_size = 12.0 * scale_y;
            let position = Point2 {
                x: (size_points.width as f32 - 30.0) * scale_x,
                y: 2.0 * scale_y,
            };

            command_list.text("blogger_sans",
                              color::BLACK,
                              text,
                              font_size,
                              position,
                              screen_matrix);

            let debug_controls = DebugControls {
                is_wireframe: self.state.is_wireframe,
                is_showing_star_field: self.state.is_showing_star_field,
                is_limiting_fps: self.state.is_limiting_fps,
                is_ui_capturing_mouse: self.state.is_ui_capturing_mouse,
                planet_subdivs: self.state.planet_subdivs as i32,
                planet_radius: self.state.planet_radius,
                star_field_radius: self.state.star_field_radius,
            };

            command_list.ui(move |ui| debug_controls.render(ui));
        }

        command_list
    }
}
