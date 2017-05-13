extern crate cgmath;
#[cfg(test)]
extern crate expectest;
extern crate find_folder;
extern crate fnv;
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
use cgmath::{Matrix4, Point2, Rad, Vector2, Vector3};
use find_folder::Search as FolderSearch;
use geomath::GeoPoint;
use std::sync::mpsc::Sender;

use engine::{Application, FrameMetrics, Loop, RenderData};
use engine::camera::ComputedCamera;
use engine::color;
use engine::input::Event as InputEvent;
use engine::math::Size2;
use engine::render::{CommandList, ResourcesRef};

use self::camera::{FirstPersonCamera, TurntableCamera};
use self::job::Job;

mod camera;
mod job;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    Close,
    SetLimitingFps(bool),
    SetCameraMode(CameraMode),
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

impl From<InputEvent> for Event {
    fn from(src: InputEvent) -> Event {
        use engine::input::ElementState::{Pressed, Released};
        use engine::input::MouseButton;
        use engine::input::VirtualKeyCode as Key;

        match src {
            InputEvent::Closed |
            InputEvent::KeyboardInput(Pressed, _, Some(Key::Escape)) => Event::Close,
            InputEvent::KeyboardInput(Pressed, _, Some(Key::R)) => Event::ResetState,
            InputEvent::KeyboardInput(Pressed, _, Some(Key::U)) => Event::ToggleUi,
            InputEvent::MouseInput(Pressed, MouseButton::Left) => Event::DragStart,
            InputEvent::MouseInput(Released, MouseButton::Left) => Event::DragEnd,
            InputEvent::MouseInput(Pressed, MouseButton::Right) => Event::ZoomStart,
            InputEvent::MouseInput(Released, MouseButton::Right) => Event::ZoomEnd,
            InputEvent::MouseMoved(x, y) => Event::MousePosition(Point2::new(x, y)),
            _ => Event::NoOp,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct StarField {
    count: usize,
    size: f32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CameraMode {
    Turntable,
    FirstPerson,
}

#[derive(Clone, Debug)]
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
    mouse_delta: Vector2<i32>,

    light_dir: Vector3<f32>,

    camera_mode: CameraMode,
    turntable_camera: TurntableCamera,
    first_person_camera: FirstPersonCamera,

    star_field_radius: f32,
    stars0: StarField,
    stars1: StarField,
    stars2: StarField,

    planet_radius: f32,
    planet_subdivs: usize,
}

impl State {
    fn new(frame_metrics: FrameMetrics) -> State {
        let planet_radius = 1.0;

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
            mouse_delta: Vector2::zero(),

            camera_mode: CameraMode::Turntable,
            turntable_camera: TurntableCamera {
                rotation: Rad(0.0),
                rotation_delta: Rad(0.0),
                zoom_delta: 0.0,
                xz_radius: 2.0,
                y_height: 1.0,
                near: 0.1,
                far: 1000.0,
                zoom_factor: 1.0,
                drag_factor: -10.0,
            },
            first_person_camera: FirstPersonCamera {
                location: GeoPoint::north(),
                height: 0.1,
                radius: planet_radius,
                near: 0.1,
                far: 1000.0,
            },

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

            planet_radius,
            planet_subdivs: 3,
        }
    }

    fn reset(&mut self) {
        *self = State::new(self.frame_metrics);
    }

    fn scene_camera(&self) -> ComputedCamera {
        let aspect_ratio = self.frame_metrics.aspect_ratio();

        match self.camera_mode {
            CameraMode::Turntable => self.turntable_camera.compute(aspect_ratio),
            CameraMode::FirstPerson => self.first_person_camera.compute(aspect_ratio),
        }
    }

    fn hud_camera(&self) -> Matrix4<f32> {
        let Size2 { width, height } = self.frame_metrics.size_pixels;
        cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
    }
}

pub struct Game {
    state: State,
    job_tx: Sender<Job>,
    resources: ResourcesRef,
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

    fn init_resources(&self) {
        use std::fs::File;
        use std::io;
        use std::io::prelude::*;
        use std::path::Path;

        let assets = FolderSearch::ParentsThenKids(3, 3)
            .for_folder("resources")
            .expect("Could not locate `resources` folder");

        fn load_shader(path: &Path) -> io::Result<String> {
            let mut file = File::open(path)?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            Ok(buffer)
        }

        self.resources
            .compile_program("flat_shaded",
                             load_shader(&assets.join("shaders/flat_shaded.v.glsl")).unwrap(),
                             load_shader(&assets.join("shaders/flat_shaded.f.glsl")).unwrap())
            .unwrap();

        self.resources
            .compile_program("text",
                             load_shader(&assets.join("shaders/text.v.glsl")).unwrap(),
                             load_shader(&assets.join("shaders/text.f.glsl")).unwrap())
            .unwrap();

        self.resources
            .compile_program("unshaded",
                             load_shader(&assets.join("shaders/unshaded.v.glsl")).unwrap(),
                             load_shader(&assets.join("shaders/unshaded.f.glsl")).unwrap())
            .unwrap();

        fn load_font(path: &Path) -> io::Result<Vec<u8>> {
            let mut file = File::open(path)?;
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;

            Ok(buffer)
        }

        self.resources
            .upload_font("blogger_sans",
                         load_font(&assets.join("fonts/blogger_sans.ttf")).unwrap())
            .unwrap();
    }
}

impl Application for Game {
    type Event = Event;

    fn init(frame_metrics: FrameMetrics, resources: ResourcesRef) -> Game {
        let game = Game {
            state: State::new(frame_metrics),
            resources: resources.clone(),
            job_tx: job_queue::spawn(move |job| Job::process(job, &resources)),
        };

        game.init_resources();
        game.queue_regenete_stars_jobs();
        game.queue_regenete_planet_job();

        game
    }

    fn handle_frame_request(&mut self, frame_metrics: FrameMetrics) -> Loop {
        self.state.frame_metrics = frame_metrics;

        match self.state.camera_mode {
            CameraMode::Turntable => {
                if self.state.is_dragging {
                    let size_points = frame_metrics.size_points;
                    let rps = self.state.mouse_delta.x as f32 / size_points.width as f32;
                    self.state.turntable_camera.rotation_delta = Rad::full_turn() * rps;
                }
                if self.state.is_zooming {
                    self.state.turntable_camera.zoom_delta = self.state.mouse_delta.y as f32;
                }

                self.state
                    .turntable_camera
                    .update(frame_metrics.delta_time);

                if self.state.is_dragging {
                    self.state.turntable_camera.rotation_delta = Rad(0.0);
                }
                if self.state.is_zooming {
                    self.state.turntable_camera.zoom_delta = 0.0;
                }
            },
            CameraMode::FirstPerson => {
                self.state
                    .first_person_camera
                    .update(frame_metrics.delta_time);
            },
        }

        self.state.mouse_delta = Vector2::zero();

        Loop::Continue
    }

    fn handle_input(&mut self, event: Event) -> Loop {
        use self::Event::*;

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
            SetCameraMode(mode) => {
                match self.state.camera_mode {
                    CameraMode::Turntable => self.state.turntable_camera.reset_motion(),
                    CameraMode::FirstPerson => self.state.first_person_camera.reset_motion(),
                }
                self.state.camera_mode = mode;
            },
            SetUiCapturingMouse(value) => self.state.is_ui_capturing_mouse = value,
            SetWireframe(value) => self.state.is_wireframe = value,
            ToggleUi => self.state.is_ui_enabled = !self.state.is_ui_enabled,
            ResetState => self.state.reset(),
            DragStart => self.state.is_dragging = !self.state.is_ui_capturing_mouse,
            DragEnd => self.state.is_dragging = false,
            ZoomStart => self.state.is_zooming = !self.state.is_ui_capturing_mouse,
            ZoomEnd => self.state.is_zooming = false,
            MousePosition(new_position) => {
                self.state.mouse_delta = new_position - self.state.mouse_position;
                self.state.mouse_position = new_position;
            },
            NoOp => {},
        }

        Loop::Continue
    }

    fn render(&self) -> RenderData<Event> {
        let mut command_list = CommandList::new();

        let camera = self.state.scene_camera();
        let screen_matrix = self.state.hud_camera();

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

            let is_wireframe = self.state.is_wireframe;
            let is_showing_star_field = self.state.is_showing_star_field;
            let is_limiting_fps = self.state.is_limiting_fps;
            let is_ui_capturing_mouse = self.state.is_ui_capturing_mouse;
            let camera_mode = self.state.camera_mode;
            let planet_subdivs = self.state.planet_subdivs as i32;
            let planet_radius = self.state.planet_radius;
            let star_field_radius = self.state.star_field_radius;

            command_list.ui(move |ui| {
                use engine::ui;

                let mut events = Vec::new();

                ui.window(im_str!("State"))
                    .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
                    .size((300.0, 250.0), imgui::ImGuiSetCond_FirstUseEver)
                    .build(|| {
                        ui::checkbox(ui, im_str!("Wireframe"), is_wireframe)
                    .map(|v| events.push(Event::SetWireframe(v)));
                        ui::checkbox(ui, im_str!("Show star field"), is_showing_star_field)
                            .map(|v| events.push(Event::SetShowingStarField(v)));
                        ui::checkbox(ui, im_str!("Limit FPS"), is_limiting_fps)
                    .map(|v| events.push(Event::SetLimitingFps(v)));

                        let camera_options = &[im_str!("Turntable"), im_str!("First Person")];
                        let camera_index = match camera_mode {
                            CameraMode::Turntable => 0,
                            CameraMode::FirstPerson => 1,
                        };
                        ui::combo(ui, im_str!("Camera mode"), camera_index, camera_options, 2)
                            .map(|v| {
                                let mode = match v {
                                    0 => CameraMode::Turntable,
                                    1 => CameraMode::FirstPerson,
                                    v => panic!("Unexpected combo index: {:?}", v),
                                };

                                events.push(Event::SetCameraMode(mode));
                            });

                        ui::slider_int(ui, im_str!("Planet subdivisions"), planet_subdivs, 1, 8)
                            .map(|v| events.push(Event::SetPlanetSubdivisions(v as usize)));
                        ui::slider_float(ui, im_str!("Planet radius"), planet_radius, 0.0, 2.0)
                            .map(|v| events.push(Event::SetPlanetRadius(v)));
                        ui::slider_float(ui,
                                         im_str!("Star field radius"),
                                         star_field_radius,
                                         0.0,
                                         20.0)
                                .map(|v| events.push(Event::SetStarFieldRadius(v)));

                        if ui.small_button(im_str!("Reset state")) {
                            events.push(Event::ResetState);
                        }
                    });

                if ui.want_capture_mouse() != is_ui_capturing_mouse {
                    events.push(Event::SetUiCapturingMouse(ui.want_capture_mouse()));
                }

                events
            });
        }

        RenderData {
            metrics: self.state.frame_metrics,
            is_limiting_fps: self.state.is_limiting_fps,
            command_list,
        }
    }
}
