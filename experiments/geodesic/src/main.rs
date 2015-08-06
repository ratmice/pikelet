#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

use glutin::Event;
use glutin::ElementState as State;
use glutin::VirtualKeyCode as KeyCode;
use glutin::{GlProfile, GlRequest, WindowBuilder};
use gfx::traits::*;

gfx_vertex!(Vertex {
    a_Pos @ pos: [f32; 2],
    a_Color @ color: [f32; 3],
});

gfx_parameters!( Params {
    foo @ foo: i32,
});

fn main() {
    let window = WindowBuilder::new()
        .with_title("Geodesic Experiment".to_string())
        .with_dimensions(800, 500)
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .build().unwrap();

    let (mut stream, mut device, mut factory) = gfx_window_glutin::init(window);

    let vertex_data = [
        Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] },
    ];
    let mesh = factory.create_mesh(&vertex_data);

    let index_data: &[u8] = &[
        0, 1,
        1, 2,
        2, 0,
    ];

    let vs = gfx::ShaderSource {
        // glsl_120: Some(include_bytes!("triangle_120.vs")),
        glsl_150: Some(include_bytes!("triangle_150.vs")),
        .. gfx::ShaderSource::empty()
    };
    let fs = gfx::ShaderSource {
        // glsl_120: Some(include_bytes!("triangle_120.fs")),
        glsl_150: Some(include_bytes!("triangle_150.fs")),
        .. gfx::ShaderSource::empty()
    };
    let program = factory.link_program_source(vs, fs).unwrap();
    let state = gfx::DrawState::new();

    let mut batch = gfx::batch::Full::new(mesh, program, Params { foo: 0, _r: std::marker::PhantomData, }).unwrap();
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
