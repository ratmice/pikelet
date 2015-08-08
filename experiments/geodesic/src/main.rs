#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra as na;

use glutin::Event;
use glutin::ElementState as State;
use glutin::VirtualKeyCode as KeyCode;
use glutin::{GlProfile, GlRequest, WindowBuilder};
use gfx::traits::*;
use na::{Iso3, Mat4, Pnt3, PerspMat3, Vec3};

gfx_vertex!(Vertex {
    a_Pos @ pos: [f32; 3],
});

gfx_parameters!(Params {
    u_Model @ model: [[f32; 4]; 4],
    u_View @ view: [[f32; 4]; 4],
    u_Proj @ proj: [[f32; 4]; 4],
});

impl<T: gfx::Resources> Params<T> {
    fn new(model: &Mat4<f32>, view: &Iso3<f32>, proj: &PerspMat3<f32>) -> Params<T> {
        Params {
            model: *model.as_array(),
            view: *na::to_homogeneous(&na::inv(view).unwrap()).as_array(),
            proj: *proj.to_mat().as_array(),
            _r: std::marker::PhantomData
        }
    }
}

fn icosahedron_points() -> [Pnt3<f32>; 12] {
    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    let du = 1.0 / f32::sqrt(phi * phi + 1.0);
    let dv = phi * du;

    [
        Pnt3::new(0.0,  dv,  du),
        Pnt3::new(0.0,  dv, -du),
        Pnt3::new(0.0, -dv,  du),
        Pnt3::new(0.0, -dv, -du),
        Pnt3::new( du, 0.0,  dv),
        Pnt3::new(-du, 0.0,  dv),
        Pnt3::new( du, 0.0, -dv),
        Pnt3::new(-du, 0.0, -dv),
        Pnt3::new( dv,  du, 0.0),
        Pnt3::new( dv, -du, 0.0),
        Pnt3::new(-dv,  du, 0.0),
        Pnt3::new(-dv, -du, 0.0),
    ]
}

fn icosahedron_edges() -> [[u8; 2]; 30] {
    [
        [ 0,  1], [ 0,  4], [ 0,  5], [ 0,  8], [ 0, 10],
        [ 1,  6], [ 1,  7], [ 1,  8], [ 1, 10], [ 2,  3],
        [ 2,  4], [ 2,  5], [ 2,  9], [ 2, 11], [ 3,  6],
        [ 3,  7], [ 3,  9], [ 3, 11], [ 4,  5], [ 4,  8],
        [ 4,  9], [ 5, 10], [ 5, 11], [ 6,  7], [ 6,  8],
        [ 6,  9], [ 7, 10], [ 7, 11], [ 8,  9], [10, 11],
    ]
}

fn main() {
    let window = WindowBuilder::new()
        .with_title("Geodesic Experiment".to_string())
        .with_dimensions(800, 500)
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .build().unwrap();

    let (w, h) = window.get_inner_size().unwrap();

    let (mut stream, mut device, mut factory) = gfx_window_glutin::init(window);

    let vertex_data: Vec<_> = icosahedron_points().iter()
        .map(|p| Vertex { pos: *p.as_array() })
        .collect();

    let index_data: Vec<_> = icosahedron_edges().iter()
        .flat_map(|is| is.iter())
        .map(|i| *i)
        .collect();

    let vs = gfx::ShaderSource {
        glsl_150: Some(include_bytes!("triangle_150.v.glsl")),
        .. gfx::ShaderSource::empty()
    };

    let fs = gfx::ShaderSource {
        glsl_150: Some(include_bytes!("triangle_150.f.glsl")),
        .. gfx::ShaderSource::empty()
    };

    let program = factory.link_program_source(vs, fs).unwrap();
    let mesh = factory.create_mesh(&vertex_data);

    let model = na::one::<Mat4<f32>>();
    let mut view = na::one::<Iso3<f32>>();
    view.look_at_z(&Pnt3::new(3.0, 3.0, 3.0), &na::orig(), &Vec3::z());

    let aspect = w as f32 / h as f32;
    let fov = 45.0 * (std::f32::consts::PI / 180.0);
    let proj = PerspMat3::new(aspect, fov, 0.1, 300.0);

    let params = Params::new(&model, &view, &proj);

    let mut batch = gfx::batch::Full::new(mesh, program, params).unwrap();
    batch.slice = index_data.to_slice(&mut factory, gfx::PrimitiveType::Line);
    batch.state = batch.state.depth(gfx::state::Comparison::LessEqual, true);

    'main: loop {
        for event in stream.out.window.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(State::Pressed, _, Some(KeyCode::Escape)) => break 'main,
                _ => {},
            }
        }

        stream.clear(gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        });
        stream.draw(&batch).unwrap();
        stream.present(&mut device);
    }
}