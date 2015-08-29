#[macro_use]
extern crate glium;
extern crate nalgebra as na;
extern crate vtime;

use glium::{Surface, IndexBuffer, VertexBuffer};
use glium::glutin::Event;
use glium::glutin::ElementState as State;
use glium::glutin::VirtualKeyCode as KeyCode;
use glium::glutin::{GlProfile, GlRequest, WindowBuilder};
use glium::index::PrimitiveType;
use na::{Mat4, Pnt3};

mod camera;
mod color;
mod icosahedron;
mod math;

use camera::Camera;

#[derive(Copy, Clone)]
struct Vertex {
    position: Pnt3<f32>,
}

implement_vertex!(Vertex, position);

impl Vertex {
    fn icosahedron() -> Vec<Vertex> {
        icosahedron::points().iter()
            .map(|p| Vertex { position: *p })
            .collect()
    }
}

struct Model {
    color: color::Color,
    model: Mat4<f32>,
    index_buffer: IndexBuffer<u8>,
    vertex_buffer: VertexBuffer<Vertex>,
}

impl Model {
    fn draw(&self, target: &mut glium::Frame, program: &glium::Program,
            camera_mat: &Mat4<f32>, draw_params: &glium::DrawParameters,
    ) -> Result<(), glium::DrawError> {
        let uniforms = uniform! {
            color: self.color,
            model: self.model,
            camera: *camera_mat,
        };
        target.draw(&self.vertex_buffer, &self.index_buffer, program, &uniforms, draw_params)
    }
}

fn get_aspect_ratio(display: &glium::Display) -> f32 {
    let (w, h) = display.get_framebuffer_dimensions();
    w as f32 / h as f32
}

fn flatten_slices<'a, T, Slice, It>(it: It) -> Vec<T> where
    T: 'a + Clone,
    It: Iterator<Item = Slice>,
    Slice: IntoIterator<Item = &'a T, IntoIter = std::slice::Iter<'a, T>>,
{
    it.flat_map(IntoIterator::into_iter).map(Clone::clone).collect()
}

fn main() {
    use glium::DisplayBuild;

    let display = WindowBuilder::new()
        .with_title("Geodesic Experiment".to_string())
        .with_dimensions(800, 500)
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let program = program!(&display,
        150 => {
            vertex: include_str!("triangle_150.v.glsl"),
            fragment: include_str!("triangle_150.f.glsl"),
        },
    ).unwrap();

    let mut camera = Camera {
        position: Pnt3::new(5.0, 5.0, 5.0),
        ..camera::DEFAULT
    };

    let vertex_data = Vertex::icosahedron();
    let edge_indices = flatten_slices(icosahedron::edges().iter());
    let face_indices = flatten_slices(icosahedron::faces().iter());

    let wireframe = Model {
        color: color::BLACK,
        model: math::scale_mat4(1.002), // Scaled to prevent depth-fighting,
        index_buffer: IndexBuffer::new(&display, PrimitiveType::LinesList, &edge_indices).unwrap(),
        vertex_buffer: VertexBuffer::new(&display, &vertex_data).unwrap(),
    };

    let faces = Model {
        color: color::WHITE,
        model: na::one(),
        index_buffer: IndexBuffer::new(&display, PrimitiveType::TrianglesList, &face_indices).unwrap(),
        vertex_buffer: VertexBuffer::new(&display, &vertex_data).unwrap(),
    };

    'main: for time in vtime::seconds() {
        for event in display.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(State::Pressed, _, Some(KeyCode::Escape)) => break 'main,
                _ => {},
            }
        }

        // Update camera
        camera.position.x = f32::sin(time.current() as f32) * 5.0;
        camera.position.y = f32::cos(time.current() as f32) * 5.0;
        camera.aspect_ratio = get_aspect_ratio(&display);
        let camera_mat = camera.to_mat();

        // Draw params
        let draw_params = glium::DrawParameters {
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        // Draw the frame
        let mut target = display.draw();
        target.clear_color_and_depth(color::DARK_GREY, 1.0);
        faces.draw(&mut target, &program, &camera_mat, &draw_params).unwrap();
        wireframe.draw(&mut target, &program, &camera_mat, &draw_params).unwrap();
        target.finish().unwrap();
    }
}
