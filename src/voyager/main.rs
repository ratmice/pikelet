extern crate gl;
// TODO: Why can't I isolate this in platform_glfw.rs?
extern crate glfw;
extern crate native;

mod platform_glfw;

fn main() {
    let platform = platform_glfw::init();

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
