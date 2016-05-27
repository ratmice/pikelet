extern crate cgmath;
extern crate find_folder;
#[macro_use] extern crate glium;
#[macro_use] extern crate imgui;
extern crate notify;
extern crate num_traits;
#[macro_use] extern crate quick_error;
extern crate rand;
extern crate rayon;
extern crate rusttype;
extern crate time;
#[macro_use] extern crate itertools;

use cgmath::conv::*;
use cgmath::prelude::*;
use cgmath::{Matrix4, PerspectiveFov, Point2, Point3, Rad, Vector3};
use find_folder::Search as FolderSearch;
use glium::{Frame, IndexBuffer, Program, Surface, VertexBuffer, BackfaceCullingMode};
use glium::glutin;
use glium::index::{PrimitiveType, NoIndices};
use rayon::prelude::*;
use std::mem;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::time::Duration;

use camera::{Camera, ComputedCamera};
use geom::Mesh;
use geom::primitives;
use geom::algorithms::{Subdivide, Dual};
use geom::star_field::{Star, StarField};
use render::{Resources, RenderTarget, Vertex};
use ui::Context as UiContext;

pub mod app;
pub mod camera;
pub mod color;
pub mod geom;
pub mod math;
pub mod text;
pub mod times;
pub mod render;
pub mod ui;

pub fn create_planet_mesh(radius: f32, subdivs: usize) -> Mesh {
    primitives::icosahedron(radius)
        .subdivide(subdivs, &|a, b| math::midpoint_arc(radius, a, b))
        .generate_dual()
}

pub fn create_vertices(mesh: &Mesh) -> Vec<Vertex> {
    const VERTICES_PER_FACE: usize = 3;

    let mut vertices = Vec::with_capacity(mesh.faces.len() * VERTICES_PER_FACE);
    for face in mesh.faces.iter() {
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

fn create_star_vertices(stars: &[Star]) -> Vec<Vertex> {
    const STAR_FIELD_RADIUS: f32 = 20.0;

    let mut star_vertices = Vec::with_capacity(stars.len());
    stars.par_iter()
        .map(|star| Vertex { position: array3(star.position.to_point(STAR_FIELD_RADIUS)) })
        .collect_into(&mut star_vertices);

    star_vertices
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameData {
    window_dimensions: (u32, u32),
    hidpi_factor: f32,
    delta_time: f32,
    frames_per_second: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UpdateEvent {
    FrameRequested(FrameData),
    Input(InputEvent),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEvent {
    Close,
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
    UpdatePlanetSubdivisions(usize),
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

#[derive(Copy, Clone, Debug, PartialEq)]
enum ResourceEvent {
    RegeneratePlanet { radius: f32, subdivs: usize },
}

#[derive(Clone, Debug, PartialEq)]
enum Loop {
    Continue,
    Break,
}

struct State {
    delta_time: f32,
    frames_per_second: f32,

    is_wireframe: bool,
    is_showing_star_field: bool,
    is_showing_ui: bool,
    is_dragging: bool,
    is_ui_capturing_mouse: bool,
    is_zooming: bool,

    window_title: String,
    mouse_position: Point2<i32>,
    window_dimensions: (u32, u32),
    hidpi_factor: f32,

    light_dir: Vector3<f32>,

    camera_rotation: Rad<f32>,
    camera_rotation_delta: Rad<f32>,
    camera_xz_radius: f32,
    camera_y_height: f32,
    camera_near: f32,
    camera_far: f32,
    camera_zoom_factor: f32,
    camera_drag_factor: f32,

    star_field: StarField,
    star0_size: f32,
    star1_size: f32,
    star2_size: f32,

    planet_radius: f32,
    planet_subdivs: usize,
}

impl State {
    fn new() -> State {
        State {
            delta_time: 0.0,
            frames_per_second: 0.0,

            is_wireframe: false,
            is_showing_star_field: true,
            is_showing_ui: true,
            is_dragging: false,
            is_ui_capturing_mouse: false,
            is_zooming: false,

            light_dir: Vector3::new(0.0, 1.0, 0.2),

            window_title: "Geodesic Test".to_string(),
            mouse_position: Point2::origin(),
            window_dimensions: (1000, 500),
            hidpi_factor: 1.0,

            camera_rotation: Rad::new(0.0),
            camera_rotation_delta: Rad::new(0.0),
            camera_xz_radius: 2.0,
            camera_y_height: 1.0,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_zoom_factor: 10.0,
            camera_drag_factor: 10.0,

            star_field: StarField::generate(),
            star0_size: 1.0,
            star1_size: 2.5,
            star2_size: 5.0,

            planet_radius: 1.0,
            planet_subdivs: 3,
        }
    }

    fn init_display(&self) -> glium::Display {
        use glium::DisplayBuild;
        use glium::glutin::WindowBuilder;

        let (width, height) = self.window_dimensions;

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

        let flat_shaded_program = Program::from_source(display, &flat_shaded_vert, &flat_shaded_frag, None).unwrap();
        let text_program        = Program::from_source(display, &text_vert, &text_frag, None).unwrap();
        let unshaded_program    = Program::from_source(display, &unshaded_vert, &unshaded_frag, None).unwrap();

        let blogger_sans_font = {
            let mut file = File::open(assets.join("fonts/blogger/Blogger Sans.ttf")).unwrap();
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).unwrap();

            let font_collection = FontCollection::from_bytes(buffer);
            font_collection.into_font().unwrap()
        };

        let planet_mesh = create_planet_mesh(self.planet_radius, self.planet_subdivs);

        Resources {
            context: display.get_context().clone(),

            planet_vertex_buffer: VertexBuffer::new(display, &create_vertices(&planet_mesh)).unwrap(),
            index_buffer: NoIndices(PrimitiveType::TrianglesList),

            text_vertex_buffer: VertexBuffer::new(display, &text::TEXTURE_VERTICES).unwrap(),
            text_index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &text::TEXTURE_INDICES).unwrap(),

            stars0_vertex_buffer: VertexBuffer::new(display, &create_star_vertices(&self.star_field.stars0)).unwrap(),
            stars1_vertex_buffer: VertexBuffer::new(display, &create_star_vertices(&self.star_field.stars1)).unwrap(),
            stars2_vertex_buffer: VertexBuffer::new(display, &create_star_vertices(&self.star_field.stars2)).unwrap(),

            flat_shaded_program: flat_shaded_program,
            text_program: text_program,
            unshaded_program: unshaded_program,

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
    update_rx: Receiver<UpdateEvent>,
    resource_tx: Sender<ResourceEvent>,
    state: State,
}

impl Game {
    fn new(update_rx: Receiver<UpdateEvent>, resource_tx: Sender<ResourceEvent>, state: State) -> Game {
        Game {
            update_rx: update_rx,
            resource_tx: resource_tx,
            state: state,
        }
    }

    fn handle_mouse_update(&mut self, new_position: Point2<i32>) {
        let old_position = mem::replace(&mut self.state.mouse_position, new_position);
        let mouse_delta = new_position - old_position;

        if !self.state.is_ui_capturing_mouse {
            if self.state.is_dragging {
                let (window_width, _) = self.state.window_dimensions;
                let rotations_per_second = (mouse_delta.x as f32 / window_width as f32) * self.state.camera_drag_factor;
                self.state.camera_rotation_delta = Rad::full_turn() * rotations_per_second * self.state.delta_time;
            }

            if self.state.is_zooming {
                let zoom_delta = mouse_delta.x as f32 * self.state.delta_time;
                self.state.camera_xz_radius = self.state.camera_xz_radius - (zoom_delta * self.state.camera_zoom_factor);
            }
        }
    }

    fn handle_frame_request(&mut self, frame_data: FrameData) -> Loop {
        self.state.delta_time = frame_data.delta_time;
        self.state.window_dimensions = frame_data.window_dimensions;
        self.state.hidpi_factor = frame_data.hidpi_factor;
        self.state.frames_per_second = frame_data.frames_per_second;

        self.state.camera_rotation -= self.state.camera_rotation_delta;

        if self.state.is_dragging {
            self.state.camera_rotation_delta = Rad::new(0.0);
        }

        Loop::Continue
    }

    fn handle_input(&mut self, event: InputEvent) -> Loop {
        use InputEvent::*;

        match event {
            Close => return Loop::Break,
            SetShowingStarField(value) => self.state.is_showing_star_field = value,
            SetUiCapturingMouse(value) => self.state.is_ui_capturing_mouse = value,
            SetWireframe(value) => self.state.is_wireframe = value,
            ToggleUi => self.state.is_showing_ui = !self.state.is_showing_ui,
            ResetState => self.state = State::new(),
            DragStart => if !self.state.is_ui_capturing_mouse { self.state.is_dragging = true },
            DragEnd => self.state.is_dragging = false,
            ZoomStart => self.state.is_zooming = true,
            ZoomEnd => self.state.is_zooming = false,
            MousePosition(position) => self.handle_mouse_update(position),
            UpdatePlanetSubdivisions(planet_subdivs) => {
                self.state.planet_subdivs = planet_subdivs;
                self.resource_tx.send(ResourceEvent::RegeneratePlanet {
                    radius: self.state.planet_radius,
                    subdivs: self.state.planet_subdivs,
                }).unwrap();
            },
            NoOp => {},
        }

        Loop::Continue
    }

    fn handle_update(&mut self, event: UpdateEvent) -> Loop {
        match event {
            UpdateEvent::FrameRequested(frame_data) => self.handle_frame_request(frame_data),
            UpdateEvent::Input(event) => self.handle_input(event),
        }
    }

    fn update(&mut self) -> Loop {
        while let Ok(event) = self.update_rx.try_recv() {
            if self.handle_update(event) == Loop::Break {
                return Loop::Break;
            }
        }

        Loop::Continue
    }
}

fn handle_resource_events(resources: &mut Resources, display: &glium::Display, resource_rx: &Receiver<ResourceEvent>) {
    while let Ok(event) = resource_rx.try_recv() {
        match event {
            ResourceEvent::RegeneratePlanet { radius, subdivs } => {
                let planet_mesh = create_planet_mesh(radius, subdivs);
                resources.planet_vertex_buffer = VertexBuffer::new(display, &create_vertices(&planet_mesh)).unwrap();
                resources.index_buffer = NoIndices(PrimitiveType::TrianglesList);
            },
        }
    }
}

fn render_scene(frame: &mut Frame, state: &State, resources: &Resources) {
    let frame_dimensions = frame.get_dimensions();

    let mut target = RenderTarget {
        frame: frame,
        hidpi_factor: state.hidpi_factor,
        resources: resources,
        camera: state.create_scene_camera(frame_dimensions),
        hud_matrix: state.create_hud_camera(frame_dimensions),
        culling_mode: BackfaceCullingMode::CullClockwise,
    };

    target.clear(color::BLUE);

    if state.is_showing_star_field {
        // TODO: Render centered at eye position
        target.render_points(&resources.stars0_vertex_buffer, state.star0_size, color::WHITE).unwrap();
        target.render_points(&resources.stars1_vertex_buffer, state.star1_size, color::WHITE).unwrap();
        target.render_points(&resources.stars2_vertex_buffer, state.star2_size, color::WHITE).unwrap();
    }

    if state.is_wireframe {
        target.render_lines(&resources.planet_vertex_buffer, 0.5, color::BLACK).unwrap();
    } else {
        target.render_solid(&resources.planet_vertex_buffer, state.light_dir, color::GREEN).unwrap();
    }

    // FIXME: https://github.com/Gekkio/imgui-rs/issues/17
    // target.render_hud_text(&state.frames_per_second.to_string(), 12.0, Point2::new(2.0, 2.0), color::BLACK).unwrap();
}

fn render_ui(frame: &mut Frame, ui_context: &mut UiContext, ui_renderer: &mut ui::Renderer, state: &State, update_tx: &Sender<UpdateEvent>) {
    use InputEvent::*;

    let ui = ui_context.frame(state.window_dimensions, state.delta_time);

    let send_event = |event| {
        update_tx.send(UpdateEvent::Input(event)).unwrap();
    };

    ui.window(im_str!("State"))
        .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
        .size((250.0, 350.0), imgui::ImGuiSetCond_FirstUseEver)
        .build(|| {
            ui::checkbox(&ui, im_str!("Wireframe"), state.is_wireframe)
                .map(|v| send_event(SetWireframe(v)));
            ui::checkbox(&ui, im_str!("Show star field"), state.is_showing_star_field)
                .map(|v| send_event(SetShowingStarField(v)));
            ui::slider_i32(&ui, im_str!("Planet subdivisions"), state.planet_subdivs as i32, 1, 8)
                .map(|v| send_event(UpdatePlanetSubdivisions(v as usize)));

            if ui.small_button(im_str!("Reset state")) {
                send_event(ResetState);
            }

            ui.tree_node(im_str!("State")).build(|| {
                ui.text(im_str!("delta_time: {:?}", state.delta_time));
                ui.text(im_str!("frames_per_second: {:?}", state.frames_per_second));

                ui.separator();

                ui.text(im_str!("is_wireframe: {:?}", state.is_wireframe));
                ui.text(im_str!("is_showing_star_field: {:?}", state.is_showing_star_field));
                ui.text(im_str!("is_showing_ui: {:?}", state.is_showing_ui));
                ui.text(im_str!("is_dragging: {:?}", state.is_dragging));
                ui.text(im_str!("is_ui_capturing_mouse: {:?}", state.is_ui_capturing_mouse));
                ui.text(im_str!("is_zooming: {:?}", state.is_zooming));

                ui.separator();

                ui.text(im_str!("light_dir: {:?}", state.light_dir));

                ui.separator();

                ui.text(im_str!("mouse_position: {:?}", state.mouse_position));
                ui.text(im_str!("window_dimensions: {:?}", state.window_dimensions));

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
        send_event(SetUiCapturingMouse(ui.want_capture_mouse()));
    }

    ui_renderer.render(frame, ui, state.hidpi_factor).unwrap();
}

fn main() {
    use std::sync::mpsc;

    let (resource_tx, resource_rx) = mpsc::channel();
    let (ui_tx, ui_rx) = mpsc::channel();
    let (update_tx, update_rx) = mpsc::channel();

    let state = State::new();
    let display = state.init_display();
    let mut resources = state.init_resources(&display);

    let mut game = Game::new(update_rx, resource_tx, state);

    let mut ui_context = UiContext::new(ui_rx);
    let mut ui_renderer = ui_context.init_renderer(&display).unwrap();

    for time in times::in_seconds() {
        for event in display.poll_events() {
            if game.state.is_showing_ui {
                ui_tx.send(event.clone()).unwrap()
            };

            update_tx.send(UpdateEvent::Input(InputEvent::from(event))).unwrap();
        }

        let window = display.get_window().unwrap();

        if game.state.is_showing_ui {
            ui_context.update(window.hidpi_factor());
        }

        let frame_data = FrameData {
            window_dimensions: window.get_inner_size_points().unwrap(),
            hidpi_factor: window.hidpi_factor(),
            delta_time: time.delta() as f32,
            frames_per_second: 1.0 / time.delta() as f32,
        };

        update_tx.send(UpdateEvent::FrameRequested(frame_data)).unwrap();

        match game.update() {
            Loop::Break => break,
            Loop::Continue => {
                let mut frame = display.draw();

                handle_resource_events(&mut resources, &display, &resource_rx); // TODO: move resource_rx to Resources struct
                render_scene(&mut frame, &game.state, &resources);

                if game.state.is_showing_ui {
                    render_ui(&mut frame, &mut ui_context, &mut ui_renderer, &game.state, &update_tx);
                }

                frame.finish().unwrap()
            }
        }

        thread::sleep(Duration::from_millis(10)); // battery saver ;)
    }
}
