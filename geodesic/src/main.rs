extern crate cgmath;
#[macro_use] extern crate glium;
extern crate time;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point2, Point3, Point};
use cgmath::{Vector2, Vector3, Vector};
use glium::{DisplayBuild, Frame, Program, VertexBuffer};
use glium::{DrawParameters, PolygonMode, Surface};
use glium::index::{PrimitiveType, NoIndices};
use glium::glutin::{Event, WindowBuilder};
use std::mem;
use std::thread;
use std::time::Duration;

use camera::Camera;
use geom::Geometry;

mod macros;

pub mod camera;
pub mod color;
pub mod geom;
pub mod index;
pub mod math;
pub mod times;

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

const LIGHT_DIR: Vector3<f32> = Vector3 { x: 0.0, y: 0.5, z: 1.0 };

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

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
    const MAX_FACES_PER_NODE: usize = 6;
    const VERTICES_PER_FACE: usize = 3;

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

enum Action {
    Continue,
    Break,
}

struct State {
    is_wireframe: bool,
    is_showing_mesh: bool,
    is_dragging: bool,
    is_zooming: bool,

    mouse_position: Point2<i32>,
    new_mouse_position: Option<Point2<i32>>,
    window_dimensions: (u32, u32),

    camera_rotation: Rad<f32>,
    camera_rotation_delta: Rad<f32>,
    camera_distance: f32,
}

impl State {
    fn update<Events>(&mut self, events: Events, delta_time: f32) -> Action where
        Events: Iterator<Item = Event>,
    {
        use glium::glutin::{ElementState, MouseButton};
        use glium::glutin::VirtualKeyCode as Key;

        for event in events {
            match event {
                Event::Closed => return Action::Break,
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => match key {
                    Key::W => self.is_wireframe = !self.is_wireframe,
                    Key::M => self.is_showing_mesh = !self.is_showing_mesh,
                    Key::Escape => return Action::Break,
                    _ => {},
                },
                Event::MouseInput(mouse_state, MouseButton::Left) => match mouse_state {
                    ElementState::Pressed => self.is_dragging = true,
                    ElementState::Released => self.is_dragging = false,
                },
                Event::MouseInput(mouse_state, MouseButton::Right) => match mouse_state {
                    ElementState::Pressed => self.is_zooming = true,
                    ElementState::Released => self.is_zooming = false,
                },
                Event::MouseMoved((x, y)) => self.new_mouse_position = Some(Point2::new(x, y)),
                Event::Resized(width, height) => self.window_dimensions = (width, height),
                _ => {},
            }
        }

        let mouse_position_delta = match self.new_mouse_position.take() {
            Some(new_position) => {
                let old_position = mem::replace(&mut self.mouse_position, new_position);
                new_position - old_position
            },
            None => Vector2::zero(),
        };

        if self.is_dragging {
            let (window_width, _) = self.window_dimensions;
            let rotations_per_second = -(mouse_position_delta.x as f32 / window_width as f32) * CAMERA_DRAG_FACTOR;
            self.camera_rotation_delta = Rad::full_turn() * rotations_per_second * delta_time;
        }

        self.camera_rotation = self.camera_rotation - self.camera_rotation_delta;

        if self.is_zooming {
            let zoom_delta = mouse_position_delta.x as f32 * delta_time;
            self.camera_distance = self.camera_distance - (zoom_delta * CAMERA_ZOOM_FACTOR);
        }

        Action::Continue
    }

    fn create_camera(&self, (target_width, target_height): (u32, u32)) -> Camera {
        Camera {
            position: Point3 {
                x: Rad::sin(self.camera_rotation) * self.camera_distance,
                y: Rad::cos(self.camera_rotation) * self.camera_distance,
                z: CAMERA_Y_HEIGHT,
            },
            target: Point3::origin(),
            projection: PerspectiveFov {
                aspect: target_width as f32 / target_height as f32,
                fovy: Rad::full_turn() / 6.0,
                near: CAMERA_NEAR,
                far: CAMERA_FAR,
            },
        }
    }
}

fn draw_params<'a>(polygon_mode: PolygonMode) -> DrawParameters<'a> {
    use glium::{BackfaceCullingMode, Depth, DepthTest};
    use glium::draw_parameters::{Smooth};

    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default()
        },
        polygon_mode: polygon_mode,
        line_width: Some(0.5),
        point_size: Some(5.0),
        smooth: Some(Smooth::Nicest),
        ..DrawParameters::default()
    }
}

struct Resources {
    delaunay_vertex_buffer: VertexBuffer<Vertex>,
    voronoi_vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,

    flat_shaded_program: Program,
    unshaded_program: Program,
}

fn render(state: &State, resources: &Resources, mut target: Frame) {
    let camera = state.create_camera(target.get_dimensions());
    let view_matrix = camera.view_matrix();
    let proj_matrix = camera.projection_matrix();
    let eye_position = camera.position;

    let draw_unshaded = |target: &mut Frame, vertex_buffer, color, polygon_mode| {
        target.draw(
            vertex_buffer,
            &resources.index_buffer,
            &resources.unshaded_program,
            &uniform! {
                color:      color,
                model:      math::array_m4(Matrix4::from_scale(1.025)),
                view:       math::array_m4(view_matrix),
                proj:       math::array_m4(proj_matrix),
            },
            &draw_params(polygon_mode),
        )
    };

    let draw_flat_shaded = |target: &mut Frame, vertex_buffer, color, polygon_mode| {
        target.draw(
            vertex_buffer,
            &resources.index_buffer,
            &resources.flat_shaded_program,
            &uniform! {
                color:      color,
                light_dir:  math::array_v3(LIGHT_DIR),
                model:      math::array_m4(Matrix4::identity()),
                view:       math::array_m4(view_matrix),
                proj:       math::array_m4(proj_matrix),
                eye:        math::array_p3(eye_position),
            },
            &draw_params(polygon_mode),
        )
    };

    target.clear_color_and_depth(color::BLUE, 1.0);

    if state.is_showing_mesh {
        draw_unshaded(&mut target, &resources.delaunay_vertex_buffer,
                      color::RED, PolygonMode::Point).unwrap();
        draw_unshaded(&mut target, &resources.voronoi_vertex_buffer,
                      color::YELLOW, PolygonMode::Point).unwrap();
        draw_unshaded(&mut target, &resources.voronoi_vertex_buffer,
                      color::WHITE, PolygonMode::Line).unwrap();
    }

    let polygon_mode = if state.is_wireframe { PolygonMode::Line } else { PolygonMode::Fill };
    draw_flat_shaded(&mut target, &resources.delaunay_vertex_buffer,
                     color::GREEN, polygon_mode).unwrap();

    target.finish().unwrap();
}

fn main() {
    let display = WindowBuilder::new()
        .with_title(WINDOW_TITLE.to_string())
        .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let mut state = State {
        is_wireframe: false,
        is_showing_mesh: true,
        is_dragging: false,
        is_zooming: false,

        mouse_position: Point2::origin(),
        new_mouse_position: None,
        window_dimensions: (WINDOW_WIDTH, WINDOW_HEIGHT),

        camera_rotation: Rad::new(0.0),
        camera_rotation_delta: Rad::new(0.0),
        camera_distance: CAMERA_XZ_RADIUS,
    };

    let resources = {
        let planet = geom::icosahedron().subdivide(POLYHEDRON_SUBDIVS);
        let delaunay_vertex_buffer = VertexBuffer::new(&display, &create_delaunay_vertices(&planet)).unwrap();
        let voronoi_vertex_buffer = VertexBuffer::new(&display, &create_voronoi_vertices(&planet)).unwrap();
        let index_buffer = NoIndices(PrimitiveType::TrianglesList);

        let flat_shaded_program =
            Program::from_source(&display,
                                 include_str!("shader/flat_shaded.v.glsl"),
                                 include_str!("shader/flat_shaded.f.glsl"),
                                 None).unwrap();

        let unshaded_program =
            Program::from_source(&display,
                                 include_str!("shader/unshaded.v.glsl"),
                                 include_str!("shader/unshaded.f.glsl"),
                                 None).unwrap();

        Resources {
            delaunay_vertex_buffer: delaunay_vertex_buffer,
            voronoi_vertex_buffer: voronoi_vertex_buffer,
            index_buffer: index_buffer,

            flat_shaded_program: flat_shaded_program,
            unshaded_program: unshaded_program,
        }
    };

    for time in times::in_seconds() {
        // if let Some(window) = display.get_window() {
        //     window.set_title(&format!("{} | FPS: {:.2}",  WINDOW_TITLE, 1.0 / time.delta()));
        // }

        match state.update(display.poll_events(), time.delta() as f32) {
            Action::Break => break,
            Action::Continue => render(&state, &resources, display.draw()),
        }

        thread::sleep(Duration::from_millis(10)); // battery saver ;)
    }
}
