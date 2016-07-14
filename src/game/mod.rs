use cgmath;
use cgmath::prelude::*;
use cgmath::{Matrix4, PerspectiveFov, Point2, Point3, Rad, Vector3};
use find_folder::Search as FolderSearch;
use glium::{self, glutin};
use imgui::{self, Ui};
use std::sync::mpsc::{Receiver, Sender, SyncSender};

use {FrameMetrics, RenderData, UpdateEvent};
use camera::{Camera, ComputedCamera};
use color;
use math::Size2;
use render::{CommandList, ResourceEvent, Resources};
use ui;

use self::job::Job;

mod job;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Loop {
    Continue,
    Break,
}

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
            Event::Closed | Event::KeyboardInput(Pressed, _, Some(Key::Escape)) => InputEvent::Close,
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

            camera_rotation: Rad::new(0.0),
            camera_rotation_delta: Rad::new(0.0),
            camera_xz_radius: 2.0,
            camera_y_height: 1.0,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_zoom_factor: 10.0,
            camera_drag_factor: 10.0,

            star_field_radius: 10.0,
            stars0: StarField { size: 1.0, count: 100000 },
            stars1: StarField { size: 2.5, count: 10000 },
            stars2: StarField { size: 5.0, count: 1000 },

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
        }.compute()
    }

    fn create_hud_camera(&self) -> Matrix4<f32> {
        let Size2 { width, height } = self.frame_metrics.size_pixels;
        cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
    }
}

struct Game {
    state: State,
    job_tx: Sender<Job>,
}

impl Game {
    fn new(frame_metrics: FrameMetrics, job_tx: Sender<Job>) -> Game {
        Game {
            state: State::new(frame_metrics),
            job_tx: job_tx,
        }
    }

    fn queue_job(&self, job: Job) {
        self.job_tx.send(job).expect("Failed queue job");
    }

    fn queue_regenete_planet_job(&self) {
        self.queue_job(Job::Planet {
            subdivs: self.state.planet_subdivs,
        });
    }

    fn queue_regenete_stars_jobs(&self) {
        self.queue_job(Job::Stars { index: 2, count: self.state.stars2.count });
        self.queue_job(Job::Stars { index: 1, count: self.state.stars1.count });
        self.queue_job(Job::Stars { index: 0, count: self.state.stars0.count });
    }

    fn handle_mouse_update(&mut self, new_position: Point2<i32>) {
        use std::mem;

        let old_position = mem::replace(&mut self.state.mouse_position, new_position);
        let mouse_delta = new_position - old_position;

        if !self.state.is_ui_capturing_mouse {
            if self.state.is_dragging {
                let size_points = self.state.frame_metrics.size_points;
                let rotations_per_second = (mouse_delta.x as f32 / size_points.width as f32) * self.state.camera_drag_factor;
                self.state.camera_rotation_delta = Rad::full_turn() * rotations_per_second * self.state.frame_metrics.delta_time;
            }

            if self.state.is_zooming {
                let zoom_delta = mouse_delta.x as f32 * self.state.frame_metrics.delta_time;
                self.state.camera_xz_radius -= zoom_delta * self.state.camera_zoom_factor;
            }
        }
    }

    fn handle_frame_request(&mut self, frame_metrics: FrameMetrics) -> Loop {
        self.state.frame_metrics = frame_metrics;
        self.state.camera_rotation -= self.state.camera_rotation_delta;

        if self.state.is_dragging {
            self.state.camera_rotation_delta = Rad::new(0.0);
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
            DragStart => if !self.state.is_ui_capturing_mouse { self.state.is_dragging = true },
            DragEnd => self.state.is_dragging = false,
            ZoomStart => self.state.is_zooming = true,
            ZoomEnd => self.state.is_zooming = false,
            MousePosition(position) => self.handle_mouse_update(position),
            NoOp => {},
        }

        Loop::Continue
    }

    fn create_ui_data(&self) -> UiData {
        UiData {
            is_wireframe: self.state.is_wireframe,
            is_showing_star_field: self.state.is_showing_star_field,
            is_limiting_fps: self.state.is_limiting_fps,
            is_ui_capturing_mouse: self.state.is_ui_capturing_mouse,
            planet_subdivs: self.state.planet_subdivs,
            planet_radius: self.state.planet_radius,
            star_field_radius: self.state.star_field_radius,
        }
    }

    fn create_command_list(&self) -> CommandList {
        let mut command_list = CommandList::new();

        let camera = self.state.create_scene_camera();
        let screen_matrix = self.state.create_hud_camera();

        command_list.clear(color::BLUE);

        if self.state.is_showing_star_field {
            let star_field_matrix =
                Matrix4::from_translation(camera.position.to_vec())
                    * Matrix4::from_scale(self.state.star_field_radius);

            command_list.points("stars0", self.state.stars0.size, color::WHITE, star_field_matrix, camera);
            command_list.points("stars1", self.state.stars1.size, color::WHITE, star_field_matrix, camera);
            command_list.points("stars2", self.state.stars2.size, color::WHITE, star_field_matrix, camera);
        }

        let planet_matrix = Matrix4::from_scale(self.state.planet_radius);

        if self.state.is_wireframe {
            command_list.lines("planet", 0.5, color::BLACK, planet_matrix, camera);
        } else {
            command_list.solid("planet", self.state.light_dir, color::GREEN, planet_matrix, camera);
        }

        if self.state.is_ui_enabled {
            let size_points = self.state.frame_metrics.size_points;
            let scale = self.state.frame_metrics.framebuffer_scale();

            let text = format!("{:.2}", self.state.frame_metrics.frames_per_second());
            let font_size = 12.0 * scale.y;
            let position = Point2 {
                x: (size_points.width as f32 - 30.0) * scale.x,
                y: 2.0 * scale.y,
            };

            command_list.text("blogger_sans", color::BLACK, text, font_size, position, screen_matrix);
        }

        command_list
    }

    fn create_render_data(&self) -> RenderData<UiData> {
        RenderData {
            metrics: self.state.frame_metrics,
            is_limiting_fps: self.state.is_limiting_fps,
            command_list: self.create_command_list(),
            ui_data: if self.state.is_ui_enabled { Some(self.create_ui_data()) } else { None },
        }
    }
}

pub fn init_resources(display: &glium::Display) -> Resources {
    use std::fs::File;
    use std::io;
    use std::io::prelude::*;
    use std::path::Path;

    let mut resources = Resources::new(display);

    let assets =
        FolderSearch::ParentsThenKids(3, 3)
            .for_folder("resources")
            .expect("Could not locate `resources` folder");

    fn load_shader(path: &Path) -> io::Result<String> {
        let mut file = try!(File::open(path));
        let mut buffer = String::new();
        try!(file.read_to_string(&mut buffer));

        Ok(buffer)
    }

    resources.handle_event(ResourceEvent::CompileProgram {
        name: "flat_shaded".to_string(),
        vertex_shader: load_shader(&assets.join("shaders/flat_shaded.v.glsl")).unwrap(),
        fragment_shader: load_shader(&assets.join("shaders/flat_shaded.f.glsl")).unwrap(),
    });

    resources.handle_event(ResourceEvent::CompileProgram {
        name: "text".to_string(),
        vertex_shader: load_shader(&assets.join("shaders/text.v.glsl")).unwrap(),
        fragment_shader: load_shader(&assets.join("shaders/text.f.glsl")).unwrap(),
    });

    resources.handle_event(ResourceEvent::CompileProgram {
        name: "unshaded".to_string(),
        vertex_shader: load_shader(&assets.join("shaders/unshaded.v.glsl")).unwrap(),
        fragment_shader: load_shader(&assets.join("shaders/unshaded.f.glsl")).unwrap(),
    });

    fn load_font(path: &Path) -> io::Result<Vec<u8>> {
        let mut file = try!(File::open(path));
        let mut buffer = vec![];
        try!(file.read_to_end(&mut buffer));

        Ok(buffer)
    }

    resources.handle_event(ResourceEvent::UploadFont {
        name: "blogger_sans".to_string(),
        data: load_font(&assets.join("fonts/blogger_sans.ttf")).unwrap(),
    });

    resources
}

pub struct UiData {
    is_wireframe: bool,
    is_showing_star_field: bool,
    is_limiting_fps: bool,
    is_ui_capturing_mouse: bool,
    planet_subdivs: usize,
    planet_radius: f32,
    star_field_radius: f32,
}

pub fn run_ui<F>(ui: &Ui, state: UiData, send: F) where F: Fn(InputEvent) {
    use self::InputEvent::*;

    ui.window(im_str!("State"))
        .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
        .size((300.0, 250.0), imgui::ImGuiSetCond_FirstUseEver)
        .build(|| {
            ui::checkbox(ui, im_str!("Wireframe"), state.is_wireframe)
                .map(|v| send(SetWireframe(v)));
            ui::checkbox(ui, im_str!("Show star field"), state.is_showing_star_field)
                .map(|v| send(SetShowingStarField(v)));
            ui::checkbox(ui, im_str!("Limit FPS"), state.is_limiting_fps)
                .map(|v| send(SetLimitingFps(v)));
            ui::slider_int(ui, im_str!("Planet subdivisions"), state.planet_subdivs as i32, 1, 8)
                .map(|v| send(SetPlanetSubdivisions(v as usize)));
            ui::slider_float(ui, im_str!("Planet radius"), state.planet_radius, 0.0, 2.0)
                .map(|v| send(SetPlanetRadius(v)));
            ui::slider_float(ui, im_str!("Star field radius"), state.star_field_radius, 0.0, 20.0)
                .map(|v| send(SetStarFieldRadius(v)));

            if ui.small_button(im_str!("Reset state")) {
                send(ResetState);
            }
        });

    if ui.want_capture_mouse() != state.is_ui_capturing_mouse {
        send(SetUiCapturingMouse(ui.want_capture_mouse()));
    }
}

pub fn spawn(frame_data: FrameMetrics,
             render_tx: SyncSender<RenderData<UiData>>)
             -> (Sender<UpdateEvent<InputEvent>>, Receiver<ResourceEvent>) {
    use job_queue;
    use std::sync::mpsc;
    use std::thread;

    let (update_tx, update_rx) = mpsc::channel();
    let (job_tx, resource_rx) = job_queue::spawn(Job::process);

    thread::spawn(move || {
        let mut game = Game::new(frame_data, job_tx);

        game.queue_regenete_stars_jobs();
        game.queue_regenete_planet_job();

        for event in update_rx.iter() {
            let loop_result = match event {
                UpdateEvent::FrameRequested(frame_data) => {
                    // We send the data for the last frame so that the renderer
                    // can get started doing it's job in parallel!
                    render_tx.send(game.create_render_data())
                        .expect("Failed to send render data");

                    game.handle_frame_request(frame_data)
                },
                UpdateEvent::Input(event) => {
                    game.handle_input(event)
                },
            };

            if loop_result == Loop::Break { break };
        }
    });

    (update_tx, resource_rx)
}
