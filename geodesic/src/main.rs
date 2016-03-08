extern crate cgmath;
#[macro_use] extern crate glium;
extern crate time;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point2, Point3, Point};
use cgmath::{Vector2, Vector3, Vector};
use glium::{DisplayBuild, DrawParameters, PolygonMode, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType, NoIndices};
use glium::glutin::{ElementState, Event, WindowBuilder, MouseButton};
use glium::glutin::VirtualKeyCode as Key;
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

fn create_camera(rotation: Rad<f32>, (width, height): (u32, u32), radius: f32) -> Camera {
    Camera {
        position: Point3 {
            x: Rad::sin(rotation) * radius,
            y: Rad::cos(rotation) * radius,
            z: CAMERA_Y_HEIGHT,
        },
        target: Point3::origin(),
        projection: PerspectiveFov {
            aspect: width as f32 / height as f32,
            fovy: Rad::full_turn() / 6.0,
            near: CAMERA_NEAR,
            far: CAMERA_FAR,
        },
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

enum DragState {
    Released { drag_delta: Vector2<i32> },
    Dragging,
}

impl DragState {
    fn new() -> DragState {
        DragState::Released { drag_delta: Vector2::zero() }
    }

    fn begin_drag(&mut self) {
        *self = DragState::Dragging;
    }

    fn end_drag(&mut self, mouse_delta: Vector2<i32>) {
        if let DragState::Dragging = *self {
            *self = DragState::Released { drag_delta: mouse_delta };
        }
    }

    fn drag_delta(&self) -> Option<Vector2<i32>> {
        match *self {
            DragState::Released { drag_delta } => Some(drag_delta),
            DragState::Dragging => None,
        }
    }
}

fn main() {
    let display = WindowBuilder::new()
        .with_title(WINDOW_TITLE.to_string())
        .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    // Initialise state and resources

    let mut wireframe = false;
    let mut show_mesh = true;
    let mut is_zooming = false;
    let mut drag_state = DragState::new();

    let mut mouse_position = Point2::origin();
    let mut new_mouse_position = Point2::origin();
    let mut window_dimensions = (WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut camera_rotation = Rad::new(0.0);
    let mut cam_distance = CAMERA_XZ_RADIUS;

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

    // Main loop

    'main: for time in times::in_seconds() {
        // if let Some(window) = display.get_window() {
        //     window.set_title(&format!("{} | FPS: {:.2}",  WINDOW_TITLE, 1.0 / time.delta()));
        // }

        // Update state

        let mouse_delta = new_mouse_position - mouse_position;
        mouse_position = new_mouse_position;

        camera_rotation = {
            let drag_delta = -drag_state.drag_delta().unwrap_or(mouse_delta);

            let rotations_per_second = (drag_delta.x as f32 / window_dimensions.0 as f32) * CAMERA_DRAG_FACTOR;
            let rotation_delta = Rad::full_turn() * rotations_per_second * time.delta() as f32;

            camera_rotation - rotation_delta
        };

        if is_zooming {
            let zoom_delta = mouse_delta.x as f32 * time.delta() as f32;
            cam_distance = cam_distance - (zoom_delta * CAMERA_ZOOM_FACTOR);
        }

        // Render scene

        {
            let mut target = display.draw();
            let camera = create_camera(camera_rotation, target.get_dimensions(), cam_distance);
            let view_matrix = camera.view_matrix();
            let proj_matrix = camera.projection_matrix();
            let eye_position = camera.position;

            target.clear_color_and_depth(color::BLUE, 1.0);

            if show_mesh {
                let scaled = Matrix4::from_scale(1.025);
                target.draw(&delaunay_vertex_buffer, &index_buffer, &unshaded_program,
                        &uniform! {
                            color:      color::RED,
                            model:      math::array_m4(scaled),
                            view:       math::array_m4(view_matrix),
                            proj:       math::array_m4(proj_matrix),
                        },
                        &draw_params(PolygonMode::Point)).unwrap();

                target.draw(&voronoi_vertex_buffer, &index_buffer, &unshaded_program,
                            &uniform! {
                                color:      color::YELLOW,
                                model:      math::array_m4(scaled),
                                view:       math::array_m4(view_matrix),
                                proj:       math::array_m4(proj_matrix),
                            },
                            &draw_params(PolygonMode::Point)).unwrap();

                target.draw(&voronoi_vertex_buffer, &index_buffer, &unshaded_program,
                            &uniform! {
                                color:      color::WHITE,
                                model:      math::array_m4(scaled),
                                view:       math::array_m4(view_matrix),
                                proj:       math::array_m4(proj_matrix),
                            },
                            &draw_params(PolygonMode::Line)).unwrap();
            }

            let polygon_mode = if wireframe { PolygonMode::Line } else { PolygonMode::Fill };
            target.draw(&delaunay_vertex_buffer, &index_buffer, &flat_shaded_program,
                        &uniform! {
                            color:      color::GREEN,
                            light_dir:  math::array_v3(LIGHT_DIR),
                            model:      math::array_m4(Matrix4::identity()),
                            view:       math::array_m4(view_matrix),
                            proj:       math::array_m4(proj_matrix),
                            eye:        math::array_p3(eye_position),
                        },
                        &draw_params(polygon_mode)).unwrap();

            target.finish().unwrap();
        }

        // Event handling

        for ev in display.poll_events() {
            match ev {
                Event::Closed => break 'main,
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => match key {
                    Key::W => wireframe = !wireframe,
                    Key::M => show_mesh = !show_mesh,
                    Key::Escape => break 'main,
                    _ => {},
                },
                Event::MouseInput(state, MouseButton::Left) => match state {
                    ElementState::Pressed => drag_state.begin_drag(),
                    ElementState::Released => drag_state.end_drag(mouse_delta),
                },
                Event::MouseInput(state, MouseButton::Right) => match state {
                    ElementState::Pressed => is_zooming = true,
                    ElementState::Released => is_zooming = false,
                },
                Event::MouseMoved((x, y)) => new_mouse_position = Point2::new(x, y),
                Event::Resized(width, height) => window_dimensions = (width, height),
                _ => {},
            }
        }

        thread::sleep(Duration::from_millis(10)); // battery saver ;)
    }
}
