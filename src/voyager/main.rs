
extern crate gl;
extern crate glfw;
extern crate native;

use glfw::Context;

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

fn main() {
  let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  // Choose a GL profile that is compatible with OS X 10.7+
  glfw.window_hint(glfw::ContextVersion(3, 3));
  glfw.window_hint(glfw::OpenglForwardCompat(true));
  glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

  let (window, _) = glfw.create_window(800, 600, "OpenGL", glfw::Windowed)
    .expect("Failed to create GLFW window.");

  // It is essential to make the context current before calling `gl::load_with`.
  window.make_current();

  // Load the OpenGL function pointers
  gl::load_with(|s| glfw.get_proc_address(s));

  while !window.should_close() {
    // Poll events
    glfw.poll_events();

    // Clear the screen to black
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);

    // Swap buffers
    window.swap_buffers();
  }
}
