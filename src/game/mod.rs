use cgmath;
use cgmath::conv::*;
use cgmath::prelude::*;
use cgmath::{Matrix4, PerspectiveFov, Point2, Point3, Rad, Vector3};
use find_folder::Search as FolderSearch;
use glium::{self, glutin};
use imgui::{self, Ui};
use rand::{self, Rng};
use std::sync::mpsc::{Sender, SyncSender};

use {FrameData, RenderData, UpdateEvent};
use camera::{Camera, ComputedCamera};
use color;
use geom::Mesh;
use geom::primitives;
use geom::algorithms::{Subdivide, Dual};
use math::{self, GeoPoint, Size2};
use render::{CommandList, ResourceEvent, Resources, Vertex, Indices};
use ui;

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
pub struct StarField {
    count: usize,
    size: f32,
}

#[derive(Clone)]
pub struct State {
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

            star_field_radius: 10.0,
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

    fn scene_camera_projection(&self, size_pixels: Size2<u32>) -> PerspectiveFov<f32> {
        PerspectiveFov {
            aspect: size_pixels.width as f32 / size_pixels.height as f32,
            fovy: Rad::full_turn() / 6.0,
            near: self.camera_near,
            far: self.camera_far,
        }
    }

    fn create_scene_camera(&self, size_pixels: Size2<u32>) -> ComputedCamera {
        Camera {
            position: self.scene_camera_position(),
            target: Point3::origin(),
            projection: self.scene_camera_projection(size_pixels),
        }.compute()
    }

    fn create_hud_camera(&self, size_pixels: Size2<u32>) -> Matrix4<f32> {
        cgmath::ortho(0.0, size_pixels.width as f32, size_pixels.height as f32, 0.0, -1.0, 1.0)
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
                let size_points = self.frame_data.size_points;
                let rotations_per_second = (mouse_delta.x as f32 / size_points.width as f32) * self.state.camera_drag_factor;
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

    fn create_render_data(&self) -> (RenderData, CommandList) {
        let mut command_list = CommandList::new();

        let camera = self.state.create_scene_camera(self.frame_data.size_pixels);
        let screen_matrix = self.state.create_hud_camera(self.frame_data.size_pixels);

        command_list.clear(color::BLUE);

        // TODO: Render centered at eye position
        if self.state.is_showing_star_field {
            let star_field_matrix = Matrix4::from_scale(self.state.star_field_radius);

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
            let size_points = self.frame_data.size_points;
            let scale = self.frame_data.framebuffer_scale();

            let text = format!("{:.2}", self.frame_data.frames_per_second());
            let font_size = 12.0 * scale.y;
            let position = Point2 {
                x: (size_points.width as f32 - 30.0) * scale.x,
                y: 2.0 * scale.y,
            };

            command_list.text("blogger_sans", color::BLACK, text, font_size, position, screen_matrix);
        }

        let render_data = RenderData {
            frame_data: self.frame_data,
            is_limiting_fps: self.state.is_limiting_fps,
            is_ui_enabled: self.state.is_ui_enabled,
            state: self.state.clone(),
        };

        (render_data, command_list)
    }
}

#[derive(Debug)]
enum Job {
    Planet { subdivs: usize },
    Stars { index: usize, count: usize },
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
    fn generate_planet_mesh(subdivs: usize) -> Mesh {
        primitives::icosahedron(1.0)
            .subdivide(subdivs, &|a, b| math::midpoint_arc(1.0, a, b))
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

    fn create_star_vertices(count: usize) -> Vec<Vertex> {
        let mut rng = rand::weak_rng();

        (0..count).map(|_| rng.gen::<GeoPoint<f32>>())
            .map(|star| Vertex { position: array3(star.to_point(1.0)) })
            .collect()
    }

    match job {
        Job::Planet { subdivs } => {
            let mesh = generate_planet_mesh(subdivs);
            let vertices = create_vertices(&mesh);

            ResourceEvent::UploadBuffer {
                name: "planet".to_string(),
                vertices: vertices,
                indices: Indices::TrianglesList,
            }
        },
        Job::Stars { index, count } => {
            ResourceEvent::UploadBuffer {
                name: format!("stars{}", index),
                vertices: create_star_vertices(count),
                indices: Indices::Points,
            }
        },
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

pub fn run_ui<F>(ui: &Ui, render_data: &RenderData, send: F) where F: Fn(InputEvent) {
    use self::InputEvent::*;

    ui.window(im_str!("State"))
        .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
        .size((250.0, 350.0), imgui::ImGuiSetCond_FirstUseEver)
        .build(|| {
            ui::checkbox(ui, im_str!("Wireframe"), render_data.state.is_wireframe)
                .map(|v| send(SetWireframe(v)));
            ui::checkbox(ui, im_str!("Show star field"), render_data.state.is_showing_star_field)
                .map(|v| send(SetShowingStarField(v)));
            ui::checkbox(ui, im_str!("Limit FPS"), render_data.state.is_limiting_fps)
                .map(|v| send(SetLimitingFps(v)));
            ui::slider_int(ui, im_str!("Planet subdivisions"), render_data.state.planet_subdivs as i32, 1, 8)
                .map(|v| send(SetPlanetSubdivisions(v as usize)));
            ui::slider_float(ui, im_str!("Planet radius"), render_data.state.planet_radius, 0.0, 2.0)
                .map(|v| send(SetPlanetRadius(v)));
            ui::slider_float(ui, im_str!("Star field radius"), render_data.state.star_field_radius, 0.0, 20.0)
                .map(|v| send(SetStarFieldRadius(v)));

            if ui.small_button(im_str!("Reset state")) {
                send(ResetState);
            }

            ui.tree_node(im_str!("State")).build(|| {
                ui.text(im_str!("delta_time: {:?}", render_data.frame_data.delta_time));
                ui.text(im_str!("frames_per_second: {:?}", render_data.frame_data.frames_per_second()));
                ui.text(im_str!("size_points: {:?}", render_data.frame_data.size_points));
                ui.text(im_str!("size_pixels: {:?}", render_data.frame_data.size_pixels));
                ui.text(im_str!("framebuffer_scale: {:?}", render_data.frame_data.framebuffer_scale()));

                ui.separator();

                ui.text(im_str!("is_dragging: {:?}", render_data.state.is_dragging));
                ui.text(im_str!("is_limiting_fps: {:?}", render_data.state.is_limiting_fps));
                ui.text(im_str!("is_showing_star_field: {:?}", render_data.state.is_showing_star_field));
                ui.text(im_str!("is_ui_enabled: {:?}", render_data.state.is_ui_enabled));
                ui.text(im_str!("is_ui_capturing_mouse: {:?}", render_data.state.is_ui_capturing_mouse));
                ui.text(im_str!("is_wireframe: {:?}", render_data.state.is_wireframe));
                ui.text(im_str!("is_zooming: {:?}", render_data.state.is_zooming));

                ui.separator();

                ui.text(im_str!("light_dir: {:?}", render_data.state.light_dir));

                ui.separator();

                ui.text(im_str!("mouse_position: {:?}", render_data.state.mouse_position));

                ui.separator();

                ui.text(im_str!("camera_rotation: {:?}", render_data.state.camera_rotation));
                ui.text(im_str!("camera_rotation_delta: {:?}", render_data.state.camera_rotation_delta));
                ui.text(im_str!("camera_xz_radius: {:?}", render_data.state.camera_xz_radius));
            });
        });

    if ui.want_capture_mouse() != render_data.state.is_ui_capturing_mouse {
        send(SetUiCapturingMouse(ui.want_capture_mouse()));
    }
}

pub fn spawn(frame_data: FrameData,
             resource_tx: Sender<ResourceEvent>,
             render_tx: SyncSender<(RenderData, CommandList)>) -> Sender<UpdateEvent> {
    use job_queue;
    use std::sync::mpsc;
    use std::thread;

    let (update_tx, update_rx) = mpsc::channel();

    thread::spawn(move || {
        let job_tx = job_queue::spawn(move |job| {
            resource_tx.send(process_job(job)).unwrap();
        });

        let mut game = Game::new(frame_data, job_tx);

        game.queue_regenete_stars_jobs();
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

    update_tx
}
