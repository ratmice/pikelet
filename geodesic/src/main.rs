#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra as na;
extern crate vtime;

use glutin::Event;
use glutin::ElementState as State;
use glutin::VirtualKeyCode as KeyCode;
use glutin::{GlProfile, GlRequest, WindowBuilder};
use gfx::traits::*;
use gfx::state::Comparison;
use gfx::PrimitiveType as Primitive;
use gfx::batch::Full as FullBatch;
use na::{Iso3, Mat4, Pnt3, PerspMat3, Vec3};

mod color;
mod icosahedron;

gfx_vertex!(Vertex {
    a_Pos @ pos: [f32; 3],
});

impl Vertex {
    fn icosahedron() -> Vec<Vertex> {
        icosahedron::points().iter().map(|p| Vertex { pos: *p }).collect()
    }
}

gfx_parameters!(Params {
    u_Color @ color: [f32; 4],
    u_Model @ model: [[f32; 4]; 4],
    u_View @ view: [[f32; 4]; 4],
    u_Proj @ proj: [[f32; 4]; 4],
});

impl<T: gfx::Resources> Params<T> {
    fn new(color: [f32; 4], model: &Mat4<f32>, view: &Iso3<f32>, proj: &PerspMat3<f32>) -> Params<T> {
        Params {
            color: color,
            model: *model.as_array(),
            view: *na::to_homogeneous(&na::inv(view).unwrap()).as_array(),
            proj: *proj.to_mat().as_array(),
            _r: std::marker::PhantomData
        }
    }

    fn set_view(&mut self, view: &Iso3<f32>) {
        self.view = *na::to_homogeneous(&na::inv(view).unwrap()).as_array();
    }

    fn set_proj(&mut self, proj: &PerspMat3<f32>) {
        self.proj = *proj.to_mat().as_array();
    }
}

fn scale_mat4(scale: f32) -> Mat4<f32> {
    Mat4::new(
        scale, 0.0, 0.0, 0.0,
        0.0, scale, 0.0, 0.0,
        0.0, 0.0, scale, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

fn flatten_slices<'a, T, Slice, It>(it: It) -> Vec<T> where
    T: 'a + Clone,
    It: Iterator<Item = Slice>,
    Slice: IntoIterator<Item = &'a T, IntoIter = std::slice::Iter<'a, T>>,
{
    it.flat_map(IntoIterator::into_iter).map(Clone::clone).collect()
}

fn main() {
    let window = WindowBuilder::new()
        .with_title("Geodesic Experiment".to_string())
        .with_dimensions(800, 500)
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .build().unwrap();

    let (mut stream, mut device, mut factory) = gfx_window_glutin::init(window);

    let program = {
        let vs = gfx::ShaderSource {
            glsl_150: Some(include_bytes!("triangle_150.v.glsl")),
            .. gfx::ShaderSource::empty()
        };

        let fs = gfx::ShaderSource {
            glsl_150: Some(include_bytes!("triangle_150.f.glsl")),
            .. gfx::ShaderSource::empty()
        };

        factory.link_program_source(vs, fs).unwrap()
    };

    let model = na::one::<Mat4<f32>>();
    let mut view = na::one::<Iso3<f32>>();
    let fov = 45.0 * (std::f32::consts::PI / 180.0);
    let mut proj = PerspMat3::new(stream.get_aspect_ratio(), fov, 0.1, 300.0);

    let vertex_data = Vertex::icosahedron();
    let mesh = factory.create_mesh(&vertex_data);

    let mut wireframe_batch = {
        let index_data = flatten_slices(icosahedron::edges().iter());
        let model = scale_mat4(1.002); // Scaled to prevent depth-fighting
        let params = Params::new(color::BLACK, &model, &view, &proj);
        let mut batch = FullBatch::new(mesh.clone(), program.clone(), params).unwrap();
        batch.slice = index_data.to_slice(&mut factory, Primitive::Line);
        batch.state = batch.state.depth(Comparison::LessEqual, true);
        batch
    };

    let mut face_batch = {
        let index_data = flatten_slices(icosahedron::faces().iter());
        let params = Params::new(color::WHITE, &model, &view, &proj);
        let mut batch = FullBatch::new(mesh.clone(), program.clone(), params).unwrap();
        batch.slice = index_data.to_slice(&mut factory, Primitive::TriangleList);
        batch.state = batch.state.depth(Comparison::LessEqual, true);
        batch
    };

    'main: for time in vtime::seconds() {
        for event in stream.out.window.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(State::Pressed, _, Some(KeyCode::Escape)) => break 'main,
                _ => {},
            }
        }

        // Update view matrix
        let x = f32::sin(time.current() as f32);
        let y = f32::cos(time.current() as f32);
        view.look_at_z(&Pnt3::new(x * 5.0, y * 5.0, 5.0), &na::orig(), &Vec3::z());
        proj.set_aspect(stream.get_aspect_ratio());

        wireframe_batch.params.set_view(&view);
        wireframe_batch.params.set_proj(&proj);

        face_batch.params.set_view(&view);
        face_batch.params.set_proj(&proj);

        stream.clear(gfx::ClearData {
            color: color::GREY,
            depth: 1.0,
            stencil: 0,
        });

        stream.draw(&face_batch).unwrap();
        stream.draw(&wireframe_batch).unwrap();
        stream.present(&mut device);
    }
}
