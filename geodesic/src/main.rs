extern crate cgmath;
#[macro_use] extern crate glium;
extern crate time;

use std::thread;
use std::time::Duration;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point3, Point, Vector3};
use glium::{DisplayBuild, DrawParameters, PolygonMode, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType, NoIndices};
use glium::glutin::{ElementState, Event, WindowBuilder};
use glium::glutin::VirtualKeyCode as Key;

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
const CAMERA_FAR: f32 = 300.0;

const POLYHEDRON_SUBDIVS: usize = 3;

const LIGHT_DIR: Vector3<f32> = Vector3 { x: 0.0, y: 0.5, z: 1.0 };
const ROTATIONS_PER_SECOND: f32 = 0.025;

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

    let mut vertices = Vec::with_capacity(geometry.nodes.len() * MAX_FACES_PER_NODE * VERTICES_PER_FACE);

    for (i, node) in geometry.nodes.iter().enumerate() {
        let midpoints: Vec<_> =
            geometry.adjacent_nodes(geom::NodeIndex(i)).iter()
                .map(|n| math::midpoint(node.position, n.position))
                .collect();

        let centroid = math::centroid(&midpoints);
        vertices.push(Vertex { position: centroid.into() });

        // let first = midpoints[0];
        // let mut prev = first;

        // for &curr in midpoints[1..].iter().chain(Some(&first)) {
        //     vertices.push(Vertex { position: centroid.into() });
        //     vertices.push(Vertex { position: curr.into() });
        //     vertices.push(Vertex { position: prev.into() });

        //     prev = curr;
        // }
    }

    vertices
}

fn create_camera(rotation: Rad<f32>, (width, height): (u32, u32)) -> Camera {
    Camera {
        position: Point3 {
            x: Rad::sin(rotation) * CAMERA_XZ_RADIUS,
            y: Rad::cos(rotation) * CAMERA_XZ_RADIUS,
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

fn draw_params<'a>(polygon_mode: PolygonMode, depth_test: bool) -> DrawParameters<'a> {
    use glium::{BackfaceCullingMode, Depth, DepthTest};
    use glium::draw_parameters::{Smooth};

    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: if depth_test {
            Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            }
        } else {
            Depth::default()
        },
        polygon_mode: polygon_mode,
        line_width: Some(0.5),
        point_size: Some(5.0),
        smooth: Some(Smooth::Nicest),
        ..DrawParameters::default()
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

    let mut show_mesh = true;
    let mut is_rotating = true;
    let mut camera_rotation = Rad::new(0.0);

    let planet = geom::icosahedron().subdivide(POLYHEDRON_SUBDIVS);
    let delaunay_vertex_buffer = VertexBuffer::new(&display, &create_delaunay_vertices(&planet)).unwrap();
    let voronoi_vertex_buffer = VertexBuffer::new(&display, &create_voronoi_vertices(&planet)).unwrap();
    let index_buffer = NoIndices(PrimitiveType::TrianglesList);

    let shaded_program =
        Program::from_source(&display,
                             include_str!("shader/shaded.v.glsl"),
                             include_str!("shader/shaded.f.glsl"),
                             None).unwrap();

    let flat_program =
        Program::from_source(&display,
                             include_str!("shader/flat.v.glsl"),
                             include_str!("shader/flat.f.glsl"),
                             None).unwrap();

    // Main loop

    'main: for time in times::in_seconds() {
        // if let Some(window) = display.get_window() {
        //     window.set_title(&format!("{} | FPS: {:.2}",  WINDOW_TITLE, 1.0 / time.delta()));
        // }

        // Update state

        if is_rotating {
            let delta = Rad::full_turn() * ROTATIONS_PER_SECOND * time.delta() as f32;
            camera_rotation = camera_rotation + delta;
        }

        // Render scene

        let mut target = display.draw();
        let camera = create_camera(camera_rotation, target.get_dimensions());
        let view_matrix = camera.view_matrix();
        let proj_matrix = camera.projection_matrix();
        let eye_position = camera.position;

        target.clear_color_and_depth(color::WARM_GREY, 1.0);

        if show_mesh {
            target.draw(&delaunay_vertex_buffer, &index_buffer, &flat_program,
                    &uniform! {
                        color:      color::PINK,
                        model:      math::array_m4(Matrix4::identity()),
                        view:       math::array_m4(view_matrix),
                        proj:       math::array_m4(proj_matrix),
                    },
                    &draw_params(PolygonMode::Point, true)).unwrap();
            
            target.draw(&voronoi_vertex_buffer, &index_buffer, &flat_program,
                        &uniform! {
                            color:      color::LIGHT_GREY,
                            model:      math::array_m4(Matrix4::from_scale(1.025)),
                            view:       math::array_m4(view_matrix),
                            proj:       math::array_m4(proj_matrix),
                        },
                        &draw_params(PolygonMode::Point, true)).unwrap();
            
            target.draw(&voronoi_vertex_buffer, &index_buffer, &flat_program,
                        &uniform! {
                            color:      color::HALF_GREY,
                            model:      math::array_m4(Matrix4::from_scale(1.025)),
                            view:       math::array_m4(view_matrix),
                            proj:       math::array_m4(proj_matrix),
                        },
                        &draw_params(PolygonMode::Line, true)).unwrap();
        }
        
        target.draw(&delaunay_vertex_buffer, &index_buffer, &shaded_program,
                    &uniform! {
                        color:      color::WHITE,
                        light_dir:  math::array_v3(LIGHT_DIR),
                        model:      math::array_m4(Matrix4::identity()),
                        view:       math::array_m4(view_matrix),
                        proj:       math::array_m4(proj_matrix),
                        eye:        math::array_p3(eye_position),
                    },
                    &draw_params(PolygonMode::Fill, true)).unwrap();

        target.finish().unwrap();

        // Event handling

        for ev in display.poll_events() {
            match ev {
                Event::Closed => break 'main,
                Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => match key {
                    Key::M => show_mesh = !show_mesh,
                    Key::Space => is_rotating = !is_rotating,
                    Key::Escape => break 'main,
                    _ => {},
                },
                _ => {},
            }
        }

        thread::sleep(Duration::from_millis(10 as u64)); // battery saver ;)
    }
}
