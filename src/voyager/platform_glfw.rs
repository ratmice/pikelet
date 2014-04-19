extern crate gl;
extern crate glfw;

use glfw::Context;
use glfw::Window;
use glfw::Glfw;

pub struct GlfwPlatform {
  glfw: Glfw,
  window: Window
}

// TODO: This should be implementing the Platform trait... 
impl GlfwPlatform {

  pub fn exit_requested(&self) -> bool {
    self.window.should_close()
  }

  pub fn process_events(&self) {
    self.glfw.poll_events();
  }

  pub fn swap(&self) {
    self.window.swap_buffers();
  }
  
}

pub fn init() -> GlfwPlatform {
  let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  glfw.window_hint(glfw::ContextVersion(3, 3));
  glfw.window_hint(glfw::OpenglForwardCompat(true));
  glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

  let (window, _) = glfw.create_window(800, 600, "OpenGL", glfw::Windowed)
    .expect("Failed to create GLFW window.");

  window.make_current();

  gl::load_with(|s| glfw.get_proc_address(s));

  GlfwPlatform {
    glfw: glfw,
    window: window
  }
}
