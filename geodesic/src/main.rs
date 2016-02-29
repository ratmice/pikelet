#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate vtime;

pub use glium::glutin;

use cgmath::{Angle, PerspectiveFov, Rad};
use cgmath::{Matrix4, SquareMatrix};
use cgmath::{Point, Point3};
use glium::{BackfaceCullingMode, Depth, DepthTest, DrawParameters};
use glium::{DisplayBuild, IndexBuffer, PolygonMode, Program, Surface, VertexBuffer};
use glium::index::{PrimitiveType};
use glutin::{ElementState, Event, GlProfile, GlRequest, WindowBuilder};
use glutin::VirtualKeyCode as Key;

use camera::Camera;
use polyhedra::octahedron;

pub mod camera;
pub mod color;
pub mod polyhedra;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

fn collect_vertices(vs: &[Point3<f32>]) -> Vec<Vertex> {
    vs.iter().map(|&p| Vertex { position: p.into() }).collect()
}

fn collect_indices<T: Clone, Ts>(xss: &[Ts]) -> Vec<T> where for<'a> &'a Ts: IntoIterator<Item = &'a T> {
    xss.iter()
        .flat_map(<_>::into_iter)
        .map(<_>::clone)
        .collect()
}

fn make_camera(time: f64, (width, height): (u32, u32), ) -> Camera {
    Camera {
        position: Point3 {
            x: f32::sin(time as f32 * 0.1) * 2.0,
            y: f32::cos(time as f32 * 0.1) * 2.0,
            z: 1.0,
        },
        target: Point3::origin(),
        projection: PerspectiveFov {
            aspect: width as f32 / height as f32,
            fovy: Rad::full_turn() / 6.0,
            near: 0.1,
            far: 300.0,
        },
    }
}

fn draw_params<'a>(polygon_mode: PolygonMode) -> DrawParameters<'a> {
    DrawParameters {
        backface_culling: BackfaceCullingMode::CullClockwise,
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        polygon_mode: polygon_mode,
        ..DrawParameters::default()
    }
}

fn main() {
    let display = WindowBuilder::new()
        .with_title("Thing".to_string())
        .with_dimensions(800, 500)
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let vertices = collect_vertices(&octahedron::points());
    let indices = collect_indices(&octahedron::faces());

    let vertex_buffer = VertexBuffer::new(&display, &vertices).unwrap();
    let index_buffer = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &indices).unwrap();

    let program = Program::from_source(&display,
                                       include_str!("shader/solid_150.v.glsl"),
                                       include_str!("shader/solid_150.f.glsl"),
                                       None).unwrap();

    'main: for time in vtime::seconds() {
        if let Some(window) = display.get_window() {
            window.set_title(&format!("FPS: {:.2}",  1.0 / time.delta()));
        }

        let mut target = display.draw();

        target.clear_color_and_depth(color::DARK_GREY, 1.0);

        let camera = make_camera(time.current(), target.get_dimensions());
        let view_proj = camera.to_mat();

        let to_array_mat: fn(Matrix4<f32>) -> [[f32; 4]; 4] = Matrix4::into;

        target.draw(&vertex_buffer, &index_buffer, &program,
                    &uniform! {
                        color: color::WHITE,
                        model: to_array_mat(Matrix4::identity()),
                        view_proj: to_array_mat(view_proj),
                    },
                    &draw_params(PolygonMode::Fill)).unwrap();

        target.draw(&vertex_buffer, &index_buffer, &program,
                    &uniform! {
                        color: color::BLACK,
                        // Scaled to prevent depth-fighting
                        model: to_array_mat(Matrix4::from_scale(1.001)),
                        view_proj: to_array_mat(view_proj),
                    },
                    &draw_params(PolygonMode::Line)).unwrap();

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => break 'main,
                Event::KeyboardInput(ElementState::Pressed, _, Some(Key::Escape)) => break 'main,
                _ => {},
            }
        }
    }
}
