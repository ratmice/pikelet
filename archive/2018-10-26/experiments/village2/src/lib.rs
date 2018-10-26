extern crate cgmath;
extern crate failure;
extern crate genmesh;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate noise;
extern crate rand;

use failure::Error;

use glutin::GlContext;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const TRIANGLE: [Vertex; 3] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.0, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

pub fn run() -> Result<(), Error> {
    use gfx::traits::{Device, FactoryExt};

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("Triangle example".to_string())
        .with_dimensions(1024, 768);

    let vs_src = include_bytes!("shader/triangle_150_core.glslv");
    let fs_src = include_bytes!("shader/triangle_150_core.glslf");

    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .with_vsync(true);
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);
    let mut encoder = gfx::Encoder::from(factory.create_command_buffer());

    let pso = factory.create_pipeline_simple(&vs_src[..], &fs_src[..], pipe::new())?;
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_color,
    };

    let mut context_error = None;

    {
        let context_error = &mut context_error;

        events_loop.run_forever(move |event| {
            use glutin::{ControlFlow, Event, VirtualKeyCode, WindowEvent};

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Closed => return ControlFlow::Break,
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => return ControlFlow::Break,
                        Some(_) | None => {},
                    },
                    WindowEvent::Resized(width, height) => {
                        window.resize(width, height);
                        gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    },
                    _ => {},
                }
            }

            // draw a frame

            encoder.clear(&data.out, CLEAR_COLOR);
            encoder.draw(&slice, &pso, &data);
            encoder.flush(&mut device);

            match window.swap_buffers() {
                Ok(()) => {
                    device.cleanup();
                    ControlFlow::Continue
                },
                Err(err) => {
                    *context_error = Some(err);
                    ControlFlow::Break
                },
            }
        });
    }

    match context_error {
        None => Ok(()),
        Some(err) => Err(Error::from(err)),
    }
}
