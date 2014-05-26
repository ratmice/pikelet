
extern crate glfw;

use self::glfw::Context;
use self::glfw::Window;
use self::glfw::Glfw;
use self::glfw::WindowEvent;

use platform::{Command, Platform, InputManager};
use platform::{SwitchState, SwitchOff, SwitchOn};
use resources::ResourceManager;

macro_rules! input_manager {
    (keys {
        $($key_field:ident: ($Key:ident, $key_name:pat)),+
    }) => (
        pub struct GlfwInputManager {
            $($key_field: Option<Box<Command<SwitchState>>>),+
        }

        impl GlfwInputManager {
            fn new() -> GlfwInputManager {
                GlfwInputManager {
                    $($key_field: None),+
                }
            }

            fn handle_key(&self, key: glfw::Key, action: glfw::Action) {
                match key {
                    $(glfw::$Key => {
                        self.$key_field.as_ref().map(|command| call_switch(*command, action));
                    }),+
                }
            }
        }

        impl InputManager for GlfwInputManager {
            fn set_switch_command(&mut self, name: &str, command: Option<Box<Command<SwitchState>>>) -> bool {
                match name {
                    $($key_name => { self.$key_field = command; true }),+
                    _ => false,
                }
            }
        }
    )
}

fn call_switch(switch: &Command<SwitchState>, action: glfw::Action) {
    match action {
        glfw::Release => switch.call(SwitchOff),
        glfw::Press   => switch.call(SwitchOn),
        glfw::Repeat  => (),
    }
}

input_manager! {
    keys {
        key_space:               (KeySpace,         "Space"),
        key_apostrophe:          (KeyApostrophe,    "Apostrophe"),
        key_comma:               (KeyComma,         "Comma"),
        key_minus:               (KeyMinus,         "Minus"),
        key_period:              (KeyPeriod,        "Period"),
        key_slash:               (KeySlash,         "Slash"),
        key_0:                   (Key0,             "0"),
        key_1:                   (Key1,             "1"),
        key_2:                   (Key2,             "2"),
        key_3:                   (Key3,             "3"),
        key_4:                   (Key4,             "4"),
        key_5:                   (Key5,             "5"),
        key_6:                   (Key6,             "6"),
        key_7:                   (Key7,             "7"),
        key_8:                   (Key8,             "8"),
        key_9:                   (Key9,             "9"),
        key_semicolon:           (KeySemicolon,     "Semicolon"),
        key_equal:               (KeyEqual,         "Equal"),
        key_a:                   (KeyA,             "A"),
        key_b:                   (KeyB,             "B"),
        key_c:                   (KeyC,             "C"),
        key_d:                   (KeyD,             "D"),
        key_e:                   (KeyE,             "E"),
        key_f:                   (KeyF,             "F"),
        key_g:                   (KeyG,             "G"),
        key_h:                   (KeyH,             "H"),
        key_i:                   (KeyI,             "I"),
        key_j:                   (KeyJ,             "J"),
        key_k:                   (KeyK,             "K"),
        key_l:                   (KeyL,             "L"),
        key_m:                   (KeyM,             "M"),
        key_n:                   (KeyN,             "N"),
        key_o:                   (KeyO,             "O"),
        key_p:                   (KeyP,             "P"),
        key_q:                   (KeyQ,             "Q"),
        key_r:                   (KeyR,             "R"),
        key_s:                   (KeyS,             "S"),
        key_t:                   (KeyT,             "T"),
        key_u:                   (KeyU,             "U"),
        key_v:                   (KeyV,             "V"),
        key_w:                   (KeyW,             "W"),
        key_x:                   (KeyX,             "X"),
        key_y:                   (KeyY,             "Y"),
        key_z:                   (KeyZ,             "Z"),
        key_left_bracket:        (KeyLeftBracket,   "LeftBracket"),
        key_backslash:           (KeyBackslash,     "Backslash"),
        key_right_bracket:       (KeyRightBracket,  "RightBracket"),
        key_grave_accent:        (KeyGraveAccent,   "GraveAccent"),
        key_world_1:             (KeyWorld1,        "World1"),
        key_world_2:             (KeyWorld2,        "World2"),
        key_escape:              (KeyEscape,        "Escape"),
        key_enter:               (KeyEnter,         "Enter"),
        key_tab:                 (KeyTab,           "Tab"),
        key_backspace:           (KeyBackspace,     "Backspace"),
        key_insert:              (KeyInsert,        "Insert"),
        key_delete:              (KeyDelete,        "Delete"),
        key_right:               (KeyRight,         "Right"),
        key_left:                (KeyLeft,          "Left"),
        key_down:                (KeyDown,          "Down"),
        key_up:                  (KeyUp,            "Up"),
        key_page_up:             (KeyPageUp,        "PageUp"),
        key_page_down:           (KeyPageDown,      "PageDown"),
        key_home:                (KeyHome,          "Home"),
        key_end:                 (KeyEnd,           "End"),
        key_caps_lock:           (KeyCapsLock,      "CapsLock"),
        key_scroll_lock:         (KeyScrollLock,    "ScrollLock"),
        key_num_lock:            (KeyNumLock,       "NumLock"),
        key_print_screen:        (KeyPrintScreen,   "PrintScreen"),
        key_pause:               (KeyPause,         "Pause"),
        key_f1:                  (KeyF1,            "F1"),
        key_f2:                  (KeyF2,            "F2"),
        key_f3:                  (KeyF3,            "F3"),
        key_f4:                  (KeyF4,            "F4"),
        key_f5:                  (KeyF5,            "F5"),
        key_f6:                  (KeyF6,            "F6"),
        key_f7:                  (KeyF7,            "F7"),
        key_f8:                  (KeyF8,            "F8"),
        key_f9:                  (KeyF9,            "F9"),
        key_f10:                 (KeyF10,           "F10"),
        key_f11:                 (KeyF11,           "F11"),
        key_f12:                 (KeyF12,           "F12"),
        key_f13:                 (KeyF13,           "F13"),
        key_f14:                 (KeyF14,           "F14"),
        key_f15:                 (KeyF15,           "F15"),
        key_f16:                 (KeyF16,           "F16"),
        key_f17:                 (KeyF17,           "F17"),
        key_f18:                 (KeyF18,           "F18"),
        key_f19:                 (KeyF19,           "F19"),
        key_f20:                 (KeyF20,           "F20"),
        key_f21:                 (KeyF21,           "F21"),
        key_f22:                 (KeyF22,           "F22"),
        key_f23:                 (KeyF23,           "F23"),
        key_f24:                 (KeyF24,           "F24"),
        key_f25:                 (KeyF25,           "F25"),
        key_kp_0:                (KeyKp0,           "NumPad0"),
        key_kp_1:                (KeyKp1,           "NumPad1"),
        key_kp_2:                (KeyKp2,           "NumPad2"),
        key_kp_3:                (KeyKp3,           "NumPad3"),
        key_kp_4:                (KeyKp4,           "NumPad4"),
        key_kp_5:                (KeyKp5,           "NumPad5"),
        key_kp_6:                (KeyKp6,           "NumPad6"),
        key_kp_7:                (KeyKp7,           "NumPad7"),
        key_kp_8:                (KeyKp8,           "NumPad8"),
        key_kp_9:                (KeyKp9,           "NumPad9"),
        key_kp_decimal:          (KeyKpDecimal,     "NumPadDecimal"),
        key_kp_divide:           (KeyKpDivide,      "NumPadDivide"),
        key_kp_multiply:         (KeyKpMultiply,    "NumPadMultiply"),
        key_kp_subtract:         (KeyKpSubtract,    "NumPadSubtract"),
        key_kp_add:              (KeyKpAdd,         "NumPadAdd"),
        key_kp_enter:            (KeyKpEnter,       "NumPadEnter"),
        key_kp_equal:            (KeyKpEqual,       "NumPadEqual"),
        key_left_shift:          (KeyLeftShift,     "LeftShift"),
        key_left_control:        (KeyLeftControl,   "LeftControl"),
        key_left_alt:            (KeyLeftAlt,       "LeftAlt"),
        key_left_super:          (KeyLeftSuper,     "LeftSuper"),
        key_right_shift:         (KeyRightShift,    "RightShift"),
        key_right_control:       (KeyRightControl,  "RightControl"),
        key_right_alt:           (KeyRightAlt,      "RightAlt"),
        key_right_super:         (KeyRightSuper,    "RightSuper"),
        key_menu:                (KeyMenu,          "Menu")
    }
}

pub struct GlfwPlatform {
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
    input: GlfwInputManager,
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
                        (glfw::KeyEscape, glfw::Press) => {
                            // use command for this
                            self.signal_shutdown()
                        },
                        _ => ()
                    }
                    self.input.handle_key(key, action);
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
    glfw.window_hint(glfw::Resizable(false));

    // Load platform configuration
    let config = resources.open_config("platform.json")
        .expect("Failed to load platform configuration!");
    println!("DEBUG: platform configuration: {}", config.to_pretty_str());

    let title = "Voyager";
    let (window, events) = match config.find(&String::from_str("video")) {
        Some(video) => {
            let width = video.find(&String::from_str("width"))
                .and_then(|w| w.as_number())
                .and_then(|w| w.to_u32())
                .unwrap_or(800);
            let height = video.find(&String::from_str("height"))
                .and_then(|h| h.as_number())
                .and_then(|h| h.to_u32())
                .unwrap_or(600);
            let is_fullscreen = video.find(&String::from_str("fullscreen"))
                .and_then(|w| w.as_boolean())
                .unwrap_or(false);

            glfw.with_primary_monitor(|m| {
                let monitor = m.expect("Failed to detect primary monitor.");

                let (win, events) = if is_fullscreen {
                    glfw.create_window(width, height, title, glfw::FullScreen(monitor))
                } else {
                    glfw.create_window(width, height, title, glfw::Windowed)
                }.expect("Failed to create GLFW window with the provided platform configuration.");

                let video_mode = monitor.get_video_mode().expect("Unable to determine video mode for primary monitor.");
                let center_x = video_mode.width / 2;
                let center_y = video_mode.height / 2;
                let x = (center_x - width/2) as i32;
                let y = (center_y - height/2) as i32;
                win.set_pos(x, y);

                (win, events)
            })
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
        events: events,
        input: GlfwInputManager::new(),
    }
}
