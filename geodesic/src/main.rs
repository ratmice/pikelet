extern crate cgmath;
#[macro_use] extern crate glium;
extern crate rand;
extern crate rayon;
extern crate rusttype;
extern crate time;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::Matrix4;
use cgmath::{Point2, Point3, Point};
use cgmath::Vector3;
use glium::{DisplayBuild, Frame, IndexBuffer, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType, NoIndices};
use rand::Rng;
use rayon::prelude::*;
use std::mem;
use std::thread;
use std::time::Duration;

use camera::{Camera, ComputedCamera};
use geom::Geometry;
use math::Polar;
use render::{Resources, RenderTarget, Vertex};

mod macros;

pub mod camera;
pub mod color;
pub mod geom;
pub mod index;
pub mod input;
pub mod math;
pub mod text;
pub mod times;
pub mod render;

const WINDOW_TITLE: &'static str = "Geodesic Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 500;

const CAMERA_XZ_RADIUS: f32 = 2.0;
const CAMERA_Y_HEIGHT: f32 = 1.0;
const CAMERA_NEAR: f32 = 0.1;
const CAMERA_FAR: f32 = 1000.0;
const CAMERA_ZOOM_FACTOR: f32 = 10.0;
const CAMERA_DRAG_FACTOR: f32 = 10.0;

const POLYHEDRON_SUBDIVS: usize = 1;

const LIGHT_DIR: Vector3<f32> = Vector3 { x: 0.0, y: 1.0, z: 0.2 };

const STAR_FIELD_RADIUS: f32 = 20.0;

const STAR0_SIZE: f32 = 1.0;
const STAR1_SIZE: f32 = 2.5;
const STAR2_SIZE: f32 = 5.0;

const STARS0_COUNT: usize = 100000;
const STARS1_COUNT: usize = 10000;
const STARS2_COUNT: usize = 1000;

macro_rules! include_resource {
    (shader: $path:expr) => { include_str!(concat!("../resources/shaders/", $path)) };
    (font: $path:expr) => { include_bytes!(concat!("../resources/fonts/", $path)) };
}

const FLAT_SHADED_VERT: &'static str = include_resource!(shader: "flat_shaded.v.glsl");
const FLAT_SHADED_FRAG: &'static str = include_resource!(shader: "flat_shaded.f.glsl");
const TEXT_VERT: &'static str = include_resource!(shader: "text.v.glsl");
const TEXT_FRAG: &'static str = include_resource!(shader: "text.f.glsl");
const UNSHADED_VERT: &'static str = include_resource!(shader: "unshaded.v.glsl");
const UNSHADED_FRAG: &'static str = include_resource!(shader: "unshaded.f.glsl");

const BLOGGER_SANS_FONT: &'static [u8] = include_resource!(font: "blogger/Blogger Sans.ttf");

pub fn create_foo_vertices(mesh: &geom::half_edge::Mesh) -> Vec<Vertex> {
    // const VERTICES_PER_FACE: usize = 3;

    let mut vertices = Vec::with_capacity(mesh.vertices.len());
    for vert in &mesh.vertices {
        let p = mesh.positions[vert.attributes.position];
        vertices.push( Vertex { position: p.into() } );
    }

    vertices
}

pub fn create_delaunay_vertices(geometry: &Geometry) -> Vec<Vertex> {
    const VERTICES_PER_FACE: usize = 3;

    let mut vertices = Vec::with_capacity(geometry.faces.len() * VERTICES_PER_FACE);

    for face in &geometry.faces {
        let n0 = index::get(&geometry.nodes, face.nodes[0]).position;
        let n1 = index::get(&geometry.nodes, face.nodes[1]).position;
        let n2 = index::get(&geometry.nodes, face.nodes[2]).position;

        vertices.push(Vertex { position: n0.into() });
        vertices.push(Vertex { position: n1.into() });
        vertices.push(Vertex { position: n2.into() });
    }

    vertices
}

pub fn create_voronoi_vertices(geometry: &Geometry) -> Vec<Vertex> {
    // const MAX_FACES_PER_NODE: usize = 6;
    // const VERTICES_PER_FACE: usize = 3;

    let mut vertices = Vec::with_capacity(geometry.faces.len());

    for face in geometry.faces.iter() {
        let n0 = index::get(&geometry.nodes, face.nodes[0]).position;
        let n1 = index::get(&geometry.nodes, face.nodes[1]).position;
        let n2 = index::get(&geometry.nodes, face.nodes[2]).position;
        let mut points = Vec::with_capacity(3);
        points.push(n0);
        points.push(n1);
        points.push(n2);
        let centroid = math::centroid(&points);
        vertices.push(Vertex { position: centroid.into() });
    }

    // for (i, node) in geometry.nodes.iter().enumerate() {
    //     let midpoints: Vec<_> =
    //         geometry.adjacent_nodes(geom::NodeIndex(i)).iter()
    //             .map(|n| math::midpoint(node.position, n.position))
    //             .collect();

    //     let centroid = math::centroid(&midpoints);
    //     vertices.push(Vertex { position: centroid.into() });

    //     let first = midpoints[0];
    //     let mut prev = first;

    //     for &curr in midpoints[1..].iter().chain(Some(&first)) {
    //         vertices.push(Vertex { position: centroid.into() });
    //         vertices.push(Vertex { position: curr.into() });
    //         vertices.push(Vertex { position: prev.into() });

    //         prev = curr;
    //     }
    // }

    vertices
}

struct Star {
    pub position: Polar<Rad<f32>>,
}

impl Star {
    fn rand_spherical<R: Rng>(rng: &mut R, radius: f32) -> Self {
        Star { position: Polar::rand_spherical(rng, radius) }
    }
}

struct StarField {
    stars0: Vec<Star>,
    stars1: Vec<Star>,
    stars2: Vec<Star>,
}

impl StarField {
    fn generate(radius: f32) -> StarField {
        let mut rng = rand::weak_rng();
        StarField {
            stars0: (0..STARS0_COUNT).map(|_| Star::rand_spherical(&mut rng, radius)).collect(),
            stars1: (0..STARS1_COUNT).map(|_| Star::rand_spherical(&mut rng, radius)).collect(),
            stars2: (0..STARS2_COUNT).map(|_| Star::rand_spherical(&mut rng, radius)).collect(),
        }
    }
}

fn create_star_vertices(stars: &[Star]) -> Vec<Vertex> {
    let mut star_vertices = Vec::with_capacity(stars.len());
    stars.par_iter()
        .map(|star| {
            let position = math::array_p3(star.position.into());
            Vertex { position: position }
        })
        .collect_into(&mut star_vertices);

    star_vertices
}

enum Loop {
    Continue,
    Break,
}

struct State {
    frames_per_second: f32,

    is_wireframe: bool,
    is_showing_mesh: bool,
    is_showing_star_field: bool,
    is_dragging: bool,
    is_zooming: bool,

    mouse_position: Point2<i32>,
    window_dimensions: (u32, u32),

    light_dir: Vector3<f32>,

    camera_rotation: Rad<f32>,
    camera_rotation_delta: Rad<f32>,
    camera_distance: f32,
}

impl State {
    fn update_mouse_position(&mut self, new_position: Point2<i32>, delta_time: f32) {
        let mouse_position_delta = {
            let old_position = mem::replace(&mut self.mouse_position, new_position);
            new_position - old_position
        };

        if self.is_dragging {
            let (window_width, _) = self.window_dimensions;
            let rotations_per_second = (mouse_position_delta.x as f32 / window_width as f32) * CAMERA_DRAG_FACTOR;
            self.camera_rotation_delta = Rad::full_turn() * rotations_per_second * delta_time;
        }

        if self.is_zooming {
            let zoom_delta = mouse_position_delta.x as f32 * delta_time;
            self.camera_distance = self.camera_distance - (zoom_delta * CAMERA_ZOOM_FACTOR);
        }
    }

    fn update<Events>(&mut self, actions: Events, delta_time: f32) -> Loop where
        Events: IntoIterator,
        Events::Item: Into<input::Event>,
    {
        if self.is_dragging {
            self.camera_rotation_delta = Rad::new(0.0);
        }

        for action in actions {
            use input::Event::*;

            match action.into() {
                CloseApp => return Loop::Break,
                ToggleMesh => self.is_showing_mesh = !self.is_showing_mesh,
                ToggleStarField => self.is_showing_star_field = !self.is_showing_star_field,
                ToggleWireframe => self.is_wireframe = !self.is_wireframe,
                DragStart => self.is_dragging = true,
                DragEnd => self.is_dragging = false,
                ZoomStart => self.is_zooming = true,
                ZoomEnd => self.is_zooming = false,
                MousePosition(position) => self.update_mouse_position(position, delta_time),
                Resize(width, height) => self.window_dimensions = (width, height),
                NoOp => {},
            }
        }

        self.frames_per_second = 1.0 / delta_time;
        self.camera_rotation = self.camera_rotation - self.camera_rotation_delta;

        Loop::Continue
    }

    fn create_scene_camera(&self, (frame_width, frame_height): (u32, u32)) -> ComputedCamera {
        Camera {
            position: Point3 {
                x: Rad::sin(self.camera_rotation) * self.camera_distance,
                y: CAMERA_Y_HEIGHT,
                z: Rad::cos(self.camera_rotation) * self.camera_distance,
            },
            target: Point3::origin(),
            projection: PerspectiveFov {
                aspect: frame_width as f32 / frame_height as f32,
                fovy: Rad::full_turn() / 6.0,
                near: CAMERA_NEAR,
                far: CAMERA_FAR,
            },
        }.compute()
    }

    fn create_hud_camera(&self, (frame_width, frame_height): (u32, u32)) -> Matrix4<f32> {
        cgmath::ortho(0.0, frame_width as f32, frame_height as f32, 0.0, -1.0, 1.0)
    }
}

fn render_scene(state: &State, resources: &Resources, frame: Frame, hidpi_factor: f32) {
    let frame_dimensions = frame.get_dimensions();

    let mut target = RenderTarget {
        frame: frame,
        hidpi_factor: hidpi_factor,
        resources: resources,
        camera: state.create_scene_camera(frame_dimensions),
        hud_matrix: state.create_hud_camera(frame_dimensions),
    };

    target.clear(color::BLUE);

    if state.is_showing_star_field {
        // TODO: Render centered at eye position
        target.render_points(&resources.stars0_vertex_buffer, STAR0_SIZE, color::WHITE);
        target.render_points(&resources.stars1_vertex_buffer, STAR1_SIZE, color::WHITE);
        target.render_points(&resources.stars2_vertex_buffer, STAR2_SIZE, color::WHITE);
    }

    if state.is_showing_mesh {
        target.render_points(&resources.delaunay_vertex_buffer, 5.0, color::RED);
        target.render_points(&resources.voronoi_vertex_buffer, 5.0, color::YELLOW);
        target.render_lines(&resources.voronoi_vertex_buffer, 0.5, color::WHITE);
    }

    if state.is_wireframe {
        target.render_lines(&resources.delaunay_vertex_buffer, 0.5, color::BLACK);
    } else {
        target.render_solid(&resources.delaunay_vertex_buffer, state.light_dir, color::GREEN);
    }

    target.render_hud_text(&state.frames_per_second.to_string(), 12.0, Point2::new(2.0, 2.0), color::BLACK);

    target.finish();
}

fn main() {
    use glium::backend::Facade;
    use glium::glutin::WindowBuilder;

    let display = WindowBuilder::new()
        .with_title(WINDOW_TITLE.to_string())
        .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let mut state = State {
        frames_per_second: 0.0,

        is_wireframe: false,
        is_showing_mesh: true,
        is_showing_star_field: false,
        is_dragging: false,
        is_zooming: false,

        light_dir: LIGHT_DIR,

        mouse_position: Point2::origin(),
        window_dimensions: (WINDOW_WIDTH, WINDOW_HEIGHT),

        camera_rotation: Rad::new(0.0),
        camera_rotation_delta: Rad::new(0.0),
        camera_distance: CAMERA_XZ_RADIUS,
    };

    let resources = {
        use rusttype::FontCollection;

        let ori_geometry = geom::icosahedron().subdivide(POLYHEDRON_SUBDIVS);
        let geometry = geom::half_edge::icosahedron(1.0);
        let star_field = StarField::generate(STAR_FIELD_RADIUS);
        let font_collection = FontCollection::from_bytes(BLOGGER_SANS_FONT);

        Resources {
            context: display.get_context().clone(),

            // delaunay_vertex_buffer: VertexBuffer::new(&display, &create_delaunay_vertices(&geometry)).unwrap(),
            delaunay_vertex_buffer: VertexBuffer::new(&display, &create_foo_vertices(&geometry)).unwrap(),
            voronoi_vertex_buffer: VertexBuffer::new(&display, &create_voronoi_vertices(&ori_geometry)).unwrap(),
            index_buffer: NoIndices(PrimitiveType::TrianglesList),

            text_vertex_buffer: VertexBuffer::new(&display, &text::TEXTURE_VERTICES).unwrap(),
            text_index_buffer: IndexBuffer::new(&display, PrimitiveType::TrianglesList, &text::TEXTURE_INDICES).unwrap(),

            stars0_vertex_buffer: VertexBuffer::new(&display, &create_star_vertices(&star_field.stars0)).unwrap(),
            stars1_vertex_buffer: VertexBuffer::new(&display, &create_star_vertices(&star_field.stars1)).unwrap(),
            stars2_vertex_buffer: VertexBuffer::new(&display, &create_star_vertices(&star_field.stars2)).unwrap(),

            flat_shaded_program: Program::from_source(&display, FLAT_SHADED_VERT, FLAT_SHADED_FRAG, None).unwrap(),
            text_program: Program::from_source(&display, TEXT_VERT, TEXT_FRAG, None).unwrap(),
            unshaded_program: Program::from_source(&display, UNSHADED_VERT, UNSHADED_FRAG, None).unwrap(),

            blogger_sans_font: font_collection.into_font().unwrap(),
        }
    };

    for time in times::in_seconds() {
        let events = display.poll_events();
        let delta_time = time.delta() as f32;

        let hidpi_factor = display.get_window()
            .map(|window| window.hidpi_factor())
            .unwrap_or(1.0);

        match state.update(events, delta_time) {
            Loop::Break => break,
            Loop::Continue => render_scene(&state, &resources, display.draw(), hidpi_factor),
        }

        thread::sleep(Duration::from_millis(10)); // battery saver ;)
    }
}
