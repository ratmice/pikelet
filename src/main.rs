#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(doc_markdown))]
#![cfg_attr(feature = "clippy", allow(new_without_default))]

extern crate cgmath;
#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
extern crate find_folder;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
#[macro_use]
extern crate maplit;
extern crate notify;
extern crate num_traits;
#[macro_use]
extern crate quick_error;
extern crate rand;
extern crate rayon;
extern crate rusttype;
extern crate time;
#[macro_use]
extern crate itertools;

extern crate job_queue;

use cgmath::conv::*;
use cgmath::prelude::*;
use cgmath::{Matrix4, PerspectiveFov, Point2, Point3, Rad, Vector3};
use find_folder::Search as FolderSearch;
use glium::Frame;
use glium::glutin;
use imgui::Ui;
use rand::Rng;
use rayon::prelude::*;
use std::mem;
use std::sync::mpsc::Sender;
use std::time::Duration;

use camera::{Camera, ComputedCamera};
use geom::Mesh;
use geom::primitives;
use geom::algorithms::{Subdivide, Dual};
use math::GeoPoint;
use resources::{Resources, Vertex, Indices};
use resources::Event as ResourceEvent;
use ui::Context as UiContext;

pub mod camera;
pub mod color;
pub mod geom;
pub mod dggs;
pub mod math;
pub mod text;
pub mod times;
pub mod render;
pub mod resources;
pub mod ui;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameData {
    size_points: (u32, u32),
    size_pixels: (u32, u32),
    delta_time: f32,
}

impl FrameData {
    fn frames_per_second(&self) -> f32 {
        match self.delta_time {
            0.0 => 0.0,
            delta_time => 1.0 / delta_time,
        }
    }

    fn scale_factor(&self) -> (f32, f32) {
        (
            if self.size_points.0 == 0 { 0.0 } else { self.size_pixels.0 as f32 / self.size_points.0 as f32 },
            if self.size_points.1 == 0 { 0.0 } else { self.size_pixels.1 as f32 / self.size_points.1 as f32 },
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    Close,
    SetLimitingFps(bool),
    SetPlanetSubdivisions(usize),
    SetShowingStarField(bool),
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

struct RenderData {
    frame_data: FrameData,
    state: State,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Loop {
    Continue,
    Break,
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
    fn new() -> State {
        State {
            is_dragging: false,
            is_limiting_fps: true,
            is_showing_star_field: true,
            is_ui_enabled: true,
            is_ui_capturing_mouse: false,
            is_wireframe: false,
            is_zooming: false,

            light_dir: Vector3::new(0.0, 1.0, 0.2),

            mouse_position: Point2::origin(),

            camera_rotation: Rad::new(0.0),
            camera_rotation_delta: Rad::new(0.0),
            camera_xz_radius: 2.0,
            camera_y_height: 1.0,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_zoom_factor: 10.0,
            camera_drag_factor: 10.0,

            star_field_radius: 20.0,
            stars0: StarField { size: 1.0, count: 100000 },
            stars1: StarField { size: 2.5, count: 10000 },
            stars2: StarField { size: 5.0, count: 1000 },

            planet_radius: 1.0,
            planet_subdivs: 3,
        }
    }

    fn reset(&mut self) {
        *self = State::new();
    }

    fn scene_camera_position(&self) -> Point3<f32> {
        Point3 {
            x: Rad::sin(self.camera_rotation) * self.camera_xz_radius,
            y: self.camera_y_height,
            z: Rad::cos(self.camera_rotation) * self.camera_xz_radius,
        }
    }

    fn scene_camera_projection(&self, (frame_width, frame_height): (u32, u32)) -> PerspectiveFov<f32> {
        PerspectiveFov {
            aspect: frame_width as f32 / frame_height as f32,
            fovy: Rad::full_turn() / 6.0,
            near: self.camera_near,
            far: self.camera_far,
        }
    }

    fn create_scene_camera(&self, frame_dimensions: (u32, u32)) -> ComputedCamera {
        Camera {
            position: self.scene_camera_position(),
            target: Point3::origin(),
            projection: self.scene_camera_projection(frame_dimensions),
        }.compute()
    }

    fn create_hud_camera(&self, (frame_width, frame_height): (u32, u32)) -> Matrix4<f32> {
        cgmath::ortho(0.0, frame_width as f32, frame_height as f32, 0.0, -1.0, 1.0)
    }
}

struct Game {
    frame_data: FrameData,
    state: State,
    job_tx: Sender<Job>,
}

impl Game {
    fn new(frame_data: FrameData, job_tx: Sender<Job>) -> Game {
        Game {
            frame_data: frame_data,
            state: State::new(),
            job_tx: job_tx,
        }
    }

    fn handle_mouse_update(&mut self, new_position: Point2<i32>) {
        let old_position = mem::replace(&mut self.state.mouse_position, new_position);
        let mouse_delta = new_position - old_position;

        if !self.state.is_ui_capturing_mouse {
            if self.state.is_dragging {
                let (window_width, _) = self.frame_data.size_points;
                let rotations_per_second = (mouse_delta.x as f32 / window_width as f32) * self.state.camera_drag_factor;
                self.state.camera_rotation_delta = Rad::full_turn() * rotations_per_second * self.frame_data.delta_time;
            }

            if self.state.is_zooming {
                let zoom_delta = mouse_delta.x as f32 * self.frame_data.delta_time;
                self.state.camera_xz_radius -= zoom_delta * self.state.camera_zoom_factor;
            }
        }
    }

    fn handle_frame_request(&mut self, frame_data: FrameData) -> Loop {
        self.frame_data = frame_data;

        self.state.camera_rotation -= self.state.camera_rotation_delta;

        if self.state.is_dragging {
            self.state.camera_rotation_delta = Rad::new(0.0);
        }

        Loop::Continue
    }

    fn queue_job(&self, job: Job) {
        self.job_tx.send(job).expect("Failed queue job");
    }

    fn queue_regenete_planet_job(&self) {
        self.queue_job(Job::Planet {
            radius: self.state.planet_radius,
            subdivs: self.state.planet_subdivs,
        });
    }

    fn queue_regenete_stars_job(&self) {
        self.queue_job(Job::Stars { index: 2, count: self.state.stars2.count, radius: self.state.star_field_radius });
        self.queue_job(Job::Stars { index: 1, count: self.state.stars1.count, radius: self.state.star_field_radius });
        self.queue_job(Job::Stars { index: 0, count: self.state.stars0.count, radius: self.state.star_field_radius });
    }

    fn handle_input(&mut self, event: InputEvent) -> Loop {
        use InputEvent::*;

        match event {
            Close => return Loop::Break,
            SetLimitingFps(value) => self.state.is_limiting_fps = value,
            SetPlanetSubdivisions(planet_subdivs) => {
                self.state.planet_subdivs = planet_subdivs;
                self.queue_regenete_planet_job();
            },
            SetShowingStarField(value) => self.state.is_showing_star_field = value,
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

    fn create_render_data(&self) -> RenderData {
        RenderData {
            frame_data: self.frame_data,
            state: self.state.clone(),
        }
    }
}

#[derive(Debug)]
enum Job {
    Planet { radius: f32, subdivs: usize },
    Stars { index: usize, count: usize, radius: f32 },
}

impl PartialEq for Job {
    fn eq(&self, other: &Job) -> bool {
        match (self, other) {
            (&Job::Planet { .. }, &Job::Planet { .. }) => true,
            (&Job::Stars { index: i, .. }, &Job::Stars { index: j, .. }) => i == j,
            (&_, &_) => false,
        }
    }
}

fn process_job(job: Job) -> ResourceEvent {
    fn generate_planet_mesh(radius: f32, subdivs: usize) -> Mesh {
        primitives::icosahedron(radius)
            .subdivide(subdivs, &|a, b| math::midpoint_arc(radius, a, b))
            .generate_dual()
    }

    fn create_vertices(mesh: &Mesh) -> Vec<Vertex> {
        const VERTICES_PER_FACE: usize = 3;

        let mut vertices = Vec::with_capacity(mesh.faces.len() * VERTICES_PER_FACE);
        for face in &mesh.faces {
            let e0 = face.root;
            let e1 = mesh.edges[e0].next;
            let e2 = mesh.edges[e1].next;

            let p0 = mesh.edges[e0].position;
            let p1 = mesh.edges[e1].position;
            let p2 = mesh.edges[e2].position;

            vertices.push(Vertex { position: mesh.positions[p0].into() });
            vertices.push(Vertex { position: mesh.positions[p1].into() });
            vertices.push(Vertex { position: mesh.positions[p2].into() });
        }

        vertices
    }

    fn generate_stars(count: usize) -> Vec<GeoPoint<f32>> {
        let mut rng = rand::weak_rng();
        (0..count).map(|_| rng.gen()).collect()
    }

    fn create_star_vertices(stars: &[GeoPoint<f32>], radius: f32) -> Vec<Vertex> {
        let mut star_vertices = Vec::with_capacity(stars.len());
        stars.par_iter()
            .map(|star| Vertex { position: array3(star.to_point(radius)) })
            .collect_into(&mut star_vertices);

        star_vertices
    }

    match job {
        Job::Planet { radius, subdivs } => {
            let mesh = generate_planet_mesh(radius, subdivs);
            let vertices = create_vertices(&mesh);

            ResourceEvent::UploadBuffer {
                name: "planet".to_string(),
                vertices: vertices,
                indices: Indices::TrianglesList,
            }
        },
        Job::Stars { index, count, radius } => {
            let stars = generate_stars(count);
            let vertices = create_star_vertices(&stars, radius);

            ResourceEvent::UploadBuffer {
                name: format!("stars{}", index),
                vertices: vertices,
                indices: Indices::Points,
            }
        },
    }
}

fn init_display<S: Into<String>>(title: S, (width, height): (u32, u32)) -> glium::Display {
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;

    WindowBuilder::new()
        .with_title(title)
        .with_dimensions(width, height)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap()
}

fn init_resources(display: &glium::Display) -> Resources {
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

fn render_scene(frame: &mut Frame, frame_data: FrameData, state: &State, resources: &Resources) {
    use render::Command;

    let camera = state.create_scene_camera(frame_data.size_points);
    let screen_matrix = state.create_hud_camera(frame_data.size_pixels);

    render::handle_command(frame, resources, Command::Clear {
        color: color::BLUE,
    }).unwrap();

    if state.is_showing_star_field {
        // TODO: Render centered at eye position

        render::handle_command(frame, resources, Command::Points {
            buffer_name: "stars0".to_string(),
            size: state.stars0.size,
            color: color::WHITE,
            model: Matrix4::identity(),
            camera: camera,
        }).unwrap();

        render::handle_command(frame, resources, Command::Points {
            buffer_name: "stars1".to_string(),
            size: state.stars1.size,
            color: color::WHITE,
            model: Matrix4::identity(),
            camera: camera,
        }).unwrap();

        render::handle_command(frame, resources, Command::Points {
            buffer_name: "stars2".to_string(),
            size: state.stars2.size,
            color: color::WHITE,
            model: Matrix4::identity(),
            camera: camera,
        }).unwrap();
    }

    if state.is_wireframe {
        render::handle_command(frame, resources, Command::Lines {
            buffer_name: "planet".to_string(),
            width: 0.5,
            color: color::BLACK,
            model: Matrix4::identity(),
            camera: camera,
        }).unwrap();
    } else {
        render::handle_command(frame, resources, Command::Solid {
            buffer_name: "planet".to_string(),
            light_dir: state.light_dir,
            color: color::GREEN,
            model: Matrix4::identity(),
            camera: camera,
        }).unwrap();
    }

    if state.is_ui_enabled {
        let (frame_width, _) = frame_data.size_points;
        let (scale_x, scale_y) = frame_data.scale_factor();

        render::handle_command(frame, resources, Command::Text {
            font_name: "blogger_sans".to_string(),
            color: color::BLACK,
            text: format!("{:.2}", frame_data.frames_per_second()),
            size: 12.0 * scale_y,
            position: Point2 { x: (frame_width as f32 - 30.0) * scale_x, y: 2.0 * scale_y },
            screen_matrix: screen_matrix,
        }).unwrap();
    }
}

fn run_ui<F>(ui: &Ui, frame_data: FrameData, state: &State, send: F) where F: Fn(InputEvent) {
    use InputEvent::*;

    ui.window(im_str!("State"))
        .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
        .size((250.0, 350.0), imgui::ImGuiSetCond_FirstUseEver)
        .build(|| {
            ui::checkbox(ui, im_str!("Wireframe"), state.is_wireframe)
                .map(|v| send(SetWireframe(v)));
            ui::checkbox(ui, im_str!("Show star field"), state.is_showing_star_field)
                .map(|v| send(SetShowingStarField(v)));
            ui::checkbox(ui, im_str!("Limit FPS"), state.is_limiting_fps)
                .map(|v| send(SetLimitingFps(v)));
            ui::slider_int(ui, im_str!("Planet subdivisions"), state.planet_subdivs as i32, 1, 8)
                .map(|v| send(SetPlanetSubdivisions(v as usize)));

            if ui.small_button(im_str!("Reset state")) {
                send(ResetState);
            }

            ui.tree_node(im_str!("State")).build(|| {
                ui.text(im_str!("delta_time: {:?}", frame_data.delta_time));
                ui.text(im_str!("frames_per_second: {:?}", frame_data.frames_per_second()));
                ui.text(im_str!("size_points: {:?}", frame_data.size_points));
                ui.text(im_str!("size_pixels: {:?}", frame_data.size_pixels));
                ui.text(im_str!("scale_factor: {:?}", frame_data.scale_factor()));

                ui.separator();

                ui.text(im_str!("is_dragging: {:?}", state.is_dragging));
                ui.text(im_str!("is_limiting_fps: {:?}", state.is_limiting_fps));
                ui.text(im_str!("is_showing_star_field: {:?}", state.is_showing_star_field));
                ui.text(im_str!("is_ui_enabled: {:?}", state.is_ui_enabled));
                ui.text(im_str!("is_ui_capturing_mouse: {:?}", state.is_ui_capturing_mouse));
                ui.text(im_str!("is_wireframe: {:?}", state.is_wireframe));
                ui.text(im_str!("is_zooming: {:?}", state.is_zooming));

                ui.separator();

                ui.text(im_str!("light_dir: {:?}", state.light_dir));

                ui.separator();

                ui.text(im_str!("mouse_position: {:?}", state.mouse_position));

                ui.separator();

                ui.text(im_str!("camera_rotation: {:?}", state.camera_rotation));
                ui.text(im_str!("camera_rotation_delta: {:?}", state.camera_rotation_delta));
                ui.text(im_str!("camera_xz_radius: {:?}", state.camera_xz_radius));

                ui.separator();

                ui.text(im_str!("planet_radius: {:?}", state.planet_radius));
                ui.text(im_str!("planet_subdivs: {:?}", state.planet_subdivs));
            });
        });

    if ui.want_capture_mouse() != state.is_ui_capturing_mouse {
        send(SetUiCapturingMouse(ui.want_capture_mouse()));
    }
}

macro_rules! try_or {
    ($e:expr, $or:expr) => {
        match $e { Ok(x) => x, Err(_) => $or }
    };
}

fn main() {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum UpdateEvent {
        FrameRequested(FrameData),
        Input(InputEvent),
    }

    fn create_frame_data(display: &glium::Display, delta_time: f32) -> FrameData {
        let window = display.get_window().unwrap();

        FrameData {
            size_points: window.get_inner_size_points().unwrap(),
            size_pixels: window.get_inner_size_pixels().unwrap(),
            delta_time: delta_time,
        }
    }

    fn render_ui(target: &mut Frame, ui_context: &mut UiContext, frame_data: FrameData, state: &State, update_tx: &Sender<UpdateEvent>) {
        ui_context.render(target, frame_data, |ui| {
            run_ui(ui, frame_data, state, |event| {
                // FIXME: could cause a panic on the slim chance that the update thread
                //  closes during ui rendering.
                update_tx.send(UpdateEvent::Input(event)).unwrap();
            });
        }).unwrap();
    }

    use std::sync::mpsc;
    use std::thread;

    let (update_tx, update_rx) = mpsc::channel();
    let (resource_tx, resource_rx) = mpsc::channel();
    let (render_tx, render_rx) = mpsc::sync_channel(1);

    let display = init_display("Voyager!", (1000, 500));
    let frame_data = create_frame_data(&display, 0.0);
    let mut resources = init_resources(&display);
    let mut ui_context = UiContext::new(&display);

    // Spawn update thread
    thread::spawn(move || {
        let job_tx = job_queue::spawn(move |job| {
            resource_tx.send(process_job(job)).unwrap();
        });

        let mut game = Game::new(frame_data, job_tx);

        game.queue_regenete_stars_job();
        game.queue_regenete_planet_job();

        for event in update_rx.iter() {
            let loop_result = match event {
                UpdateEvent::FrameRequested(frame_data) => {
                    // We send the data for the last frame so that the renderer
                    // can get started doing it's job in parallel!
                    let render_data = game.create_render_data();
                    render_tx.send(render_data).expect("Failed to send render data");

                    game.handle_frame_request(frame_data)
                },
                UpdateEvent::Input(event) => {
                    game.handle_input(event)
                },
            };

            if loop_result == Loop::Break { break };
        }
    });

    'main: for time in times::in_seconds() {
        // Swap frames with update thread
        let RenderData { state, frame_data } = {
            let frame_data = create_frame_data(&display, time.delta() as f32);
            let update_event = UpdateEvent::FrameRequested(frame_data);

            try_or!(update_tx.send(update_event), break 'main);
            try_or!(render_rx.recv(), break 'main)
        };
        ui_context.set_is_enabled(state.is_ui_enabled);

        // Get user input
        for event in display.poll_events() {
            ui_context.update(event.clone());
            let update_event = UpdateEvent::Input(InputEvent::from(event));

            try_or!(update_tx.send(update_event), break 'main);
        }

        // Update resources
        while let Ok(event) = resource_rx.try_recv() {
            resources.handle_event(event);
        }

        // Render frame
        let mut frame = display.draw();
        render_scene(&mut frame, frame_data, &state, &resources);
        render_ui(&mut frame, &mut ui_context, frame_data, &state, &update_tx);
        frame.finish().unwrap();

        if state.is_limiting_fps {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
