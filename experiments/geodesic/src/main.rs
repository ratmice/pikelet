#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra as na;
extern crate time;

use glutin::Event;
use glutin::ElementState as State;
use glutin::VirtualKeyCode as KeyCode;
use glutin::{GlProfile, GlRequest, WindowBuilder};
use gfx::traits::*;
use gfx::state::Comparison;
use gfx::PrimitiveType as Primitive;
use gfx::batch::Full as FullBatch;
use na::{Iso3, Mat4, Pnt3, PerspMat3, Vec3};

gfx_vertex!(Vertex {
    a_Pos @ pos: [f32; 3],
});

mod color {
    pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    pub const GREY: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
    pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
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

/// Generates the cartesian coordinates of a [regular iocosahedron]
/// (https://en.wikipedia.org/wiki/Regular_icosahedron) with an edge length of 2.
fn icosahedron_points() -> [Pnt3<f32>; 12] {
    // The cartesian coordinates of the iocosahedron are are described by the
    // cyclic permutations of (±ϕ, ±1, 0), where ϕ is the [Golden Ratio]
    // (https://en.wikipedia.org/wiki/Golden_ratio).

    let phi = (1.0 + f32::sqrt(5.0)) / 2.0;
    [
        Pnt3::new( phi,  1.0,  0.0),
        Pnt3::new( phi, -1.0,  0.0),
        Pnt3::new(-phi,  1.0,  0.0),
        Pnt3::new(-phi, -1.0,  0.0),
        Pnt3::new( 0.0,  phi,  1.0),
        Pnt3::new( 0.0,  phi, -1.0),
        Pnt3::new( 0.0, -phi,  1.0),
        Pnt3::new( 0.0, -phi, -1.0),
        Pnt3::new( 1.0,  0.0,  phi),
        Pnt3::new(-1.0,  0.0,  phi),
        Pnt3::new( 1.0,  0.0, -phi),
        Pnt3::new(-1.0,  0.0, -phi),
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

struct Face {
    nodes: [u8; 3],
    #[allow(dead_code)]
    edges: [u8; 3],
}

fn icosahedron_faces() -> [Face; 20] {
    [
        Face { nodes: [ 0,  1,  8], edges: [ 0,  7,  3] },
        Face { nodes: [ 0,  4,  5], edges: [ 1, 18,  2] },
        Face { nodes: [ 0,  5, 10], edges: [ 2, 21,  4] },
        Face { nodes: [ 0,  8,  4], edges: [ 3, 19,  1] },
        Face { nodes: [ 0, 10,  1], edges: [ 4,  8,  0] },
        Face { nodes: [ 1,  6,  8], edges: [ 5, 24,  7] },
        Face { nodes: [ 1,  7,  6], edges: [ 6, 23,  5] },
        Face { nodes: [ 1, 10,  7], edges: [ 8, 26,  6] },
        Face { nodes: [ 2,  3, 11], edges: [ 9, 17, 13] },
        Face { nodes: [ 2,  4,  9], edges: [10, 20, 12] },
        Face { nodes: [ 2,  5,  4], edges: [11, 18, 10] },
        Face { nodes: [ 2,  9,  3], edges: [12, 16,  9] },
        Face { nodes: [ 2, 11,  5], edges: [13, 22, 11] },
        Face { nodes: [ 3,  6,  7], edges: [14, 23, 15] },
        Face { nodes: [ 3,  7, 11], edges: [15, 27, 17] },
        Face { nodes: [ 3,  9,  6], edges: [16, 25, 14] },
        Face { nodes: [ 4,  8,  9], edges: [19, 28, 20] },
        Face { nodes: [ 5, 11, 10], edges: [22, 29, 21] },
        Face { nodes: [ 6,  9,  8], edges: [25, 28, 24] },
        Face { nodes: [ 7, 10, 11], edges: [26, 29, 27] },
    ]
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

    let vertex_data: Vec<_> = icosahedron_points().iter()
        .map(|p| Vertex { pos: *p.as_array() })
        .collect();
    let mesh = factory.create_mesh(&vertex_data);

    let mut wireframe_batch = {
        let index_data: Vec<_> = icosahedron_edges().iter()
            .flat_map(|is| is.iter())
            .map(|i| *i)
            .collect();

        // Scaled to prevent depth-fighting
        let mut model = na::one::<Mat4<f32>>();
        let scale = 1.002;
        model[(0, 0)] = scale;
        model[(1, 1)] = scale;
        model[(2, 2)] = scale;

        let params = Params::new(color::BLACK, &model, &view, &proj);
        let mut batch = FullBatch::new(mesh.clone(), program.clone(), params).unwrap();
        batch.slice = index_data.to_slice(&mut factory, Primitive::Line);
        batch.state = batch.state.depth(Comparison::LessEqual, true);
        batch
    };

    let mut face_batch = {
        let index_data: Vec<_> = icosahedron_faces().iter()
            .flat_map(|face| face.nodes.iter())
            .map(|i| *i)
            .collect();

        let params = Params::new(color::WHITE, &model, &view, &proj);
        let mut batch = FullBatch::new(mesh.clone(), program.clone(), params).unwrap();
        batch.slice = index_data.to_slice(&mut factory, Primitive::TriangleList);
        batch.state = batch.state.depth(Comparison::LessEqual, true);
        batch
    };

    'main: loop {
        for event in stream.out.window.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(State::Pressed, _, Some(KeyCode::Escape)) => break 'main,
                _ => {},
            }
        }

        // Update view matrix
        let time = time::precise_time_s() as f32;
        let x = f32::sin(time);
        let y = f32::cos(time);
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
