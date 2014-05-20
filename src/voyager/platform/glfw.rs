
extern crate glfw;

use self::glfw::Context;
use self::glfw::Window;
use self::glfw::Glfw;
use self::glfw::WindowEvent;

use platform::Platform;
use resources::ResourceManager;

pub struct GlfwPlatform {
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>
}

impl Platform for GlfwPlatform {
    fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }

    fn exit_requested(&self) -> bool {
        self.window.should_close()
    }

    fn process_events(&self) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::KeyEvent(key, _, action, _) => {
                    match (key, action) {
                        (glfw::KeyEscape, glfw::Press) => self.signal_shutdown(),
                        _ => ()
                    }
                }
                _ => ()
            }
        }
    }

    fn swap(&self) {
        self.window.swap_buffers();
    }

    fn load_gl(&self, f: fn(|&str| -> Option<extern "system" fn()>)) {
        f(|s| self.glfw.get_proc_address(s));
    }

    fn signal_shutdown(&self) {
        self.window.set_should_close(true);
    }

    fn shutdown(&self) {
        //
    }
}

pub fn init(resources: &ResourceManager) -> GlfwPlatform {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    println!("DEBUG: Initializing Voyager platform with GLFW v{}", glfw::get_version_string());

    glfw.window_hint(glfw::ContextVersion(3, 3));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));

    // Load platform configuration
    let config = resources.open_config("platform.json")
        .expect("Failed to load platform configuration!");
    println!("DEBUG: platform configuration: {}", config.to_pretty_str());

    let title = "Voyager";
    let (window, events) = match config.find(&StrBuf::from_str("video")) {
        Some(video) => {
            let width = video.find(&StrBuf::from_str("width"))
                .and_then(|w| w.as_number())
                .and_then(|w| w.to_u32())
                .unwrap_or(800);
            let height = video.find(&StrBuf::from_str("height"))
                .and_then(|h| h.as_number())
                .and_then(|h| h.to_u32())
                .unwrap_or(600);
            let is_fullscreen = video.find(&StrBuf::from_str("fullscreen"))
                .and_then(|w| w.as_boolean())
                .unwrap_or(false);

            if is_fullscreen {
                glfw.with_primary_monitor(|m| {
                    glfw.create_window(width, height, title, glfw::FullScreen(
                        m.expect("Failed to detect primary monitor.")
                    ))
                })
            } else {
                glfw.create_window(width, height, title, glfw::Windowed)
            }.expect("Failed to create GLFW window with the provided platform configuration.")
        }
        None => {
            glfw.create_window(800, 600, title, glfw::Windowed)
                .expect("Failed to create GLFW window.")
        }
    };

    window.set_key_polling(true);

    window.make_current();

    GlfwPlatform {
        glfw: glfw,
        window: window,
        events: events
    }
}
