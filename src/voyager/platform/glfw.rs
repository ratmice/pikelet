extern crate glfw;

use self::glfw::Context;
use self::glfw::Window;
use self::glfw::Glfw;

use platform::Platform;

pub struct GlfwPlatform {
    glfw: Glfw,
    window: Window
}

impl Platform for GlfwPlatform {
    fn signal_shutdown(&self) {
        self.window.set_should_close(true);
    }
    
    fn exit_requested(&self) -> bool {
        self.window.should_close()
    }

    fn process_events(&self) {
        self.glfw.poll_events();
    }

    fn swap(&self) {
        self.window.swap_buffers();
    }

    fn load_gl(&self, f: fn(|&str| -> Option<extern "system" fn()>)) {
        f(|s| self.glfw.get_proc_address(s));
    }

    fn shutdown(&self) {
        //
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

    GlfwPlatform {
        glfw: glfw,
        window: window
    }
}
