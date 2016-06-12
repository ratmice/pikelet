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
use glium::{Frame, IndexBuffer, Program, Surface, VertexBuffer, BackfaceCullingMode};
use glium::glutin;
use glium::index::{PrimitiveType, NoIndices};
use imgui::Ui;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc::Sender;
use std::time::Duration;

use camera::{Camera, ComputedCamera};
use geom::Mesh;
use geom::primitives;
use geom::algorithms::{Subdivide, Dual};
use math::GeoPoint;
use render::RenderTarget;
use resources::{Resources, Vertex};
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
    window_dimensions: (u32, u32),
    hidpi_factor: f32,
    delta_time: f32,
    frames_per_second: f32,
}

impl FrameData {
    fn new(width: u32, height: u32) -> FrameData {
        FrameData {
            window_dimensions: (width, height),
            hidpi_factor: 1.0,
            delta_time: 0.0,
            frames_per_second: 0.0,
        }
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

struct RenderData(State);

#[derive(Clone)]
enum ResourceEvent {
    Planet(Vec<Vertex>),
    Stars(usize, Vec<Vertex>),
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

#[derive(Clone)]
struct State {
    frame_data: FrameData,

    is_dragging: bool,
    is_limiting_fps: bool,
    is_showing_star_field: bool,
    is_showing_ui: bool,
    is_ui_capturing_mouse: bool,
    is_wireframe: bool,
    is_zooming: bool,

    window_title: String,
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
    star0_count: usize,
    star1_count: usize,
    star2_count: usize,
    star0_size: f32,
    star1_size: f32,
    star2_size: f32,

    planet_radius: f32,
    planet_subdivs: usize,
}

impl State {
    fn new() -> State {
        State {
            frame_data: FrameData::new(1000, 500),

            is_dragging: false,
            is_limiting_fps: true,
            is_showing_star_field: true,
            is_showing_ui: true,
            is_ui_capturing_mouse: false,
            is_wireframe: false,
            is_zooming: false,

            light_dir: Vector3::new(0.0, 1.0, 0.2),

            window_title: "Geodesic Test".to_string(),
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
            star0_count: 100000,
            star1_count: 10000,
            star2_count: 1000,
            star0_size: 1.0,
            star1_size: 2.5,
            star2_size: 5.0,

            planet_radius: 1.0,
            planet_subdivs: 3,
        }
    }

    fn reset(&mut self) {
        let frame_data = self.frame_data;

        *self = State::new();
        self.frame_data = frame_data;
    }

    fn init_display(&self) -> glium::Display {
        use glium::DisplayBuild;
        use glium::glutin::WindowBuilder;

        let (width, height) = self.frame_data.window_dimensions;

        WindowBuilder::new()
            .with_title(self.window_title.clone())
            .with_dimensions(width, height)
            .with_depth_buffer(24)
            .build_glium()
            .unwrap()
    }

    fn init_resources(&self, display: &glium::Display) -> Resources {
        use glium::backend::Facade;
        use rusttype::FontCollection;
        use std::fs::File;
        use std::io;
        use std::io::prelude::*;
        use std::path::Path;

        let assets =
            FolderSearch::ParentsThenKids(3, 3)
                .for_folder("resources")
                .expect("Could not locate `resources` folder");

        let load_shader = |assets: &Path, path| -> io::Result<String> {
            let mut file = try!(File::open(assets.join(path)));
            let mut buffer = String::new();
            try!(file.read_to_string(&mut buffer));

            Ok(buffer)
        };

        let flat_shaded_vert    = load_shader(&assets, "shaders/flat_shaded.v.glsl").unwrap();
        let flat_shaded_frag    = load_shader(&assets, "shaders/flat_shaded.f.glsl").unwrap();
        let text_vert           = load_shader(&assets, "shaders/text.v.glsl").unwrap();
        let text_frag           = load_shader(&assets, "shaders/text.f.glsl").unwrap();
        let unshaded_vert       = load_shader(&assets, "shaders/unshaded.v.glsl").unwrap();
        let unshaded_frag       = load_shader(&assets, "shaders/unshaded.f.glsl").unwrap();

        let programs = hashmap! {
            "flat_shaded".to_string() => Program::from_source(display, &flat_shaded_vert, &flat_shaded_frag, None).unwrap(),
            "text".to_string()        => Program::from_source(display, &text_vert,        &text_frag,        None).unwrap(),
            "unshaded".to_string()    => Program::from_source(display, &unshaded_vert,    &unshaded_frag,    None).unwrap(),
        };

        let blogger_sans_font = {
            let mut file = File::open(assets.join("fonts/blogger/Blogger Sans.ttf")).unwrap();
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).unwrap();

            let font_collection = FontCollection::from_bytes(buffer);
            font_collection.into_font().unwrap()
        };

        Resources {
            context: display.get_context().clone(),

            buffers: HashMap::new(),
            programs: programs,

            text_vertex_buffer: VertexBuffer::new(display, &text::TEXTURE_VERTICES).unwrap(),
            text_index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &text::TEXTURE_INDICES).unwrap(),
            blogger_sans_font: blogger_sans_font,
        }
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
    job_tx: Sender<Job>,
    state: State,
}

impl Game {
    fn new(job_tx: Sender<Job>, state: State) -> Game {
        Game {
            job_tx: job_tx,
            state: state,
        }
    }

    fn handle_mouse_update(&mut self, new_position: Point2<i32>) {
        let old_position = mem::replace(&mut self.state.mouse_position, new_position);
        let mouse_delta = new_position - old_position;

        if !self.state.is_ui_capturing_mouse {
            if self.state.is_dragging {
                let (window_width, _) = self.state.frame_data.window_dimensions;
                let rotations_per_second = (mouse_delta.x as f32 / window_width as f32) * self.state.camera_drag_factor;
                self.state.camera_rotation_delta = Rad::full_turn() * rotations_per_second * self.state.frame_data.delta_time;
            }

            if self.state.is_zooming {
                let zoom_delta = mouse_delta.x as f32 * self.state.frame_data.delta_time;
                self.state.camera_xz_radius -= zoom_delta * self.state.camera_zoom_factor;
            }
        }
    }

    fn handle_frame_request(&mut self, frame_data: FrameData) -> Loop {
        self.state.frame_data = frame_data;

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
        self.queue_job(Job::Stars { index: 2, count: self.state.star2_count, radius: self.state.star_field_radius });
        self.queue_job(Job::Stars { index: 1, count: self.state.star1_count, radius: self.state.star_field_radius });
        self.queue_job(Job::Stars { index: 0, count: self.state.star0_count, radius: self.state.star_field_radius });
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
            ToggleUi => self.state.is_showing_ui = !self.state.is_showing_ui,
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
        RenderData(self.state.clone())
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

            ResourceEvent::Planet(vertices)
        },
        Job::Stars { index, count, radius } => {
            let stars = generate_stars(count);
            let vertices = create_star_vertices(&stars, radius);

            ResourceEvent::Stars(index, vertices)
        }
    }
}

fn update_resources(display: &glium::Display, resources: &mut Resources, event: ResourceEvent) {
    match event {
        ResourceEvent::Planet(vertices) => {
            resources.buffers.insert("planet".to_string(), (
                VertexBuffer::new(display, &vertices).unwrap(),
                NoIndices(PrimitiveType::TrianglesList),
            ));
        },
        ResourceEvent::Stars(index, vertices) => {
            resources.buffers.insert(format!("stars{}", index), (
                VertexBuffer::new(display, &vertices).unwrap(),
                NoIndices(PrimitiveType::Points),
            ));
        }
    }
}

fn render_scene(frame: &mut Frame, state: &State, resources: &Resources) {
    let frame_dimensions = frame.get_dimensions();

    let mut target = RenderTarget {
        frame: frame,
        hidpi_factor: state.frame_data.hidpi_factor,
        resources: resources,
        camera: state.create_scene_camera(frame_dimensions),
        hud_matrix: state.create_hud_camera(frame_dimensions),
        culling_mode: BackfaceCullingMode::CullClockwise,
    };

    target.clear(color::BLUE);

    if state.is_showing_star_field {
        // TODO: Render centered at eye position
        resources.buffers.get("stars0").map(|buf| target.render_points(buf, state.star0_size, color::WHITE).unwrap());
        resources.buffers.get("stars1").map(|buf| target.render_points(buf, state.star1_size, color::WHITE).unwrap());
        resources.buffers.get("stars2").map(|buf| target.render_points(buf, state.star2_size, color::WHITE).unwrap());
    }

    if state.is_wireframe {
        resources.buffers.get("planet").map(|buf| target.render_lines(buf, 0.5, color::BLACK).unwrap());
    } else {
        resources.buffers.get("planet").map(|buf| target.render_solid(buf, state.light_dir, color::GREEN).unwrap());
    }

    if state.is_showing_ui {
        let (window_width, _) = state.frame_data.window_dimensions;
        let fps_text = format!("{:.2}", state.frame_data.frames_per_second);
        let fps_location = Point2::new(window_width as f32 - 30.0, 2.0);
        target.render_hud_text(&fps_text, 12.0, fps_location, color::BLACK).unwrap();
    }
}

fn run_ui<F>(ui: &Ui, state: &State, send: F) where F: Fn(InputEvent) {
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
            ui::slider_i32(ui, im_str!("Planet subdivisions"), state.planet_subdivs as i32, 1, 8)
                .map(|v| send(SetPlanetSubdivisions(v as usize)));

            if ui.small_button(im_str!("Reset state")) {
                send(ResetState);
            }

            ui.tree_node(im_str!("State")).build(|| {
                ui.text(im_str!("delta_time: {:?}", state.frame_data.delta_time));
                ui.text(im_str!("frames_per_second: {:?}", state.frame_data.frames_per_second));
                ui.text(im_str!("hidpi_factor: {:?}", state.frame_data.hidpi_factor));
                ui.text(im_str!("window_dimensions: {:?}", state.frame_data.window_dimensions));

                ui.separator();

                ui.text(im_str!("is_dragging: {:?}", state.is_dragging));
                ui.text(im_str!("is_limiting_fps: {:?}", state.is_limiting_fps));
                ui.text(im_str!("is_showing_star_field: {:?}", state.is_showing_star_field));
                ui.text(im_str!("is_showing_ui: {:?}", state.is_showing_ui));
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

    fn frame_data(window: &glium::glutin::Window, time_delta: f32) -> FrameData {
        FrameData {
            window_dimensions: window.get_inner_size_points().unwrap(),
            hidpi_factor: window.hidpi_factor(),
            delta_time: time_delta as f32,
            frames_per_second: 1.0 / time_delta as f32,
        }
    }

    fn update_ui(ui_context: &mut UiContext, state: &State, event: glutin::Event) {
        if state.is_showing_ui {
            ui_context.update(event.clone(), state.frame_data.hidpi_factor);
        }
    }

    fn render_ui(frame: &mut Frame, ui_context: &mut UiContext, ui_renderer: &mut ui::Renderer, state: &State, update_tx: &Sender<UpdateEvent>) {
        if state.is_showing_ui {
            let ui = ui_context.frame(state.frame_data.window_dimensions, state.frame_data.delta_time);

            run_ui(&ui, state, |event| {
                // FIXME: could cause a panic on the slim chance that the update thread
                //  closes during ui rendering.
                update_tx.send(UpdateEvent::Input(event)).unwrap();
            });

            ui_renderer.render(frame, ui, state.frame_data.hidpi_factor).unwrap();
        }
    }

    use std::sync::mpsc;
    use std::thread;

    let (update_tx, update_rx) = mpsc::channel();
    let (resource_tx, resource_rx) = mpsc::channel();
    let (render_tx, render_rx) = mpsc::sync_channel(1);

    let state = State::new();
    let display = state.init_display();
    let mut resources = state.init_resources(&display);
    let mut ui_context = UiContext::new();
    let mut ui_renderer = ui_context.init_renderer(&display).unwrap();

    // Spawn update thread
    thread::spawn(move || {
        let job_tx = job_queue::spawn(move |job| {
            resource_tx.send(process_job(job)).unwrap();
        });

        let mut game = Game::new(job_tx, state);

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
        let RenderData(state) = {
            let window = display.get_window().unwrap();
            let frame_data = frame_data(&window, time.delta() as f32);
            try_or!(update_tx.send(UpdateEvent::FrameRequested(frame_data)), break 'main);
            try_or!(render_rx.recv(), break 'main)
        };

        // Get user input
        for event in display.poll_events() {
            update_ui(&mut ui_context, &state, event.clone());

            let update_event = UpdateEvent::Input(InputEvent::from(event));
            try_or!(update_tx.send(update_event), break 'main);
        }

        // Update resources
        while let Ok(event) = resource_rx.try_recv() {
            update_resources(&display, &mut resources, event);
        }

        // Render frame
        let mut frame = display.draw();
        render_scene(&mut frame, &state, &resources);
        render_ui(&mut frame, &mut ui_context, &mut ui_renderer, &state, &update_tx);
        frame.finish().unwrap();

        if state.is_limiting_fps {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
