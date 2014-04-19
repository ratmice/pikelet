extern crate gl;
extern crate native;

use platform::Platform;

mod platform;

fn main() {
    let platform = platform::glfw::init();

    platform.load_gl(gl::load_with);

    while !platform.exit_requested() {
        platform.process_events();

        // Clear the screen to black
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        platform.swap();
    }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}
