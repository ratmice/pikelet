extern crate cgmath;
#[macro_use] extern crate glium;
extern crate time;

pub use glium::glutin;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point3, Point, Vector3};
use glium::{DisplayBuild, DrawParameters, PolygonMode, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType, NoIndices};
use glutin::{ElementState, Event, WindowBuilder};
use glutin::VirtualKeyCode as Key;

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
const ROTATIONS_PER_SECOND: f32 = 0.1;

#[derive(Copy, Clone)]
pub struct Vertex {
    normal: [f32; 3],
    position: [f32; 3],
}

implement_vertex!(Vertex, normal, position);

pub fn create_vertices(geometry: &Geometry) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(geometry.nodes.len() * 3);

    for face in &geometry.faces {
        let n0 = index::get(&geometry.nodes, face.nodes[0]).position;
        let n1 = index::get(&geometry.nodes, face.nodes[1]).position;
        let n2 = index::get(&geometry.nodes, face.nodes[2]).position;

        let normal = math::face_normal(n0, n1, n2);

        vertices.push(Vertex { normal: normal.into(), position: n0.into() });
        vertices.push(Vertex { normal: normal.into(), position: n1.into() });
        vertices.push(Vertex { normal: normal.into(), position: n2.into() });
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

fn draw_params<'a>(polygon_mode: PolygonMode) -> DrawParameters<'a> {
    use glium::{BackfaceCullingMode, Depth, DepthTest};

    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default()
        },
        polygon_mode: polygon_mode,
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
    let vertex_buffer = VertexBuffer::new(&display, &create_vertices(planet)).unwrap();
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
        if let Some(window) = display.get_window() {
            window.set_title(&format!("{} | FPS: {:.2}",  WINDOW_TITLE, 1.0 / time.delta()));
        }


        // Update state

        if is_rotating {
            let delta = Rad::full_turn() * ROTATIONS_PER_SECOND * time.delta() as f32;
            camera_rotation = camera_rotation + delta;
        }


        // Render scene

        let mut target = display.draw();
        let camera = create_camera(camera_rotation, target.get_dimensions());
        let view_proj = camera.to_mat();

        target.clear_color_and_depth(color::DARK_GREY, 1.0);

        target.draw(&vertex_buffer, &index_buffer, &shaded_program,
                    &uniform! {
                        color:      color::WHITE,
                        light_dir:  math::array_v3(LIGHT_DIR),
                        model:      math::array_m4(Matrix4::identity()),
                        view_proj:  math::array_m4(view_proj),
                    },
                    &draw_params(PolygonMode::Fill)).unwrap();

        if show_mesh {
            target.draw(&vertex_buffer, &index_buffer, &flat_program,
                        &uniform! {
                            color:      color::BLACK,
                            // Scaled to prevent depth-fighting
                            model:      math::array_m4(Matrix4::from_scale(1.001)),
                            view_proj:  math::array_m4(view_proj),
                        },
                        &draw_params(PolygonMode::Line)).unwrap();
        }

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
    }
}
