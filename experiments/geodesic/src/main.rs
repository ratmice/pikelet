use glutin::Event;
use glutin::ElementState as State;
use glutin::VirtualKeyCode as KeyCode;

extern crate glutin;

fn main() {
    let window = glutin::WindowBuilder::new()
        .with_title("Geodesic Experiment".to_string())
        .with_dimensions(800, 500)
        .with_gl(glutin::GlRequest::Latest)
        .with_gl_profile(glutin::GlProfile::Core)
        .build().unwrap();

    unsafe { window.make_current().unwrap() };

    for event in window.wait_events() {
        window.swap_buffers().unwrap();

        match event {
            Event::Closed => break,
            Event::KeyboardInput(State::Pressed, _, Some(KeyCode::Escape)) => break,
            _ => {},
        }
    }
}
