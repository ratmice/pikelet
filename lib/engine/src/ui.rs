use glium::glutin;
use imgui::{self, ImGui, ImGuiKey, Ui};

use FrameMetrics;

pub struct Context {
    imgui: ImGui,
    mouse_pos: (i32, i32),
    mouse_pressed: (bool, bool, bool),
    mouse_wheel: f32,
}

impl Context {
    pub fn new(mut imgui: ImGui) -> Context {
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);

        imgui.set_imgui_key(ImGuiKey::Tab, 0);
        imgui.set_imgui_key(ImGuiKey::LeftArrow, 1);
        imgui.set_imgui_key(ImGuiKey::RightArrow, 2);
        imgui.set_imgui_key(ImGuiKey::UpArrow, 3);
        imgui.set_imgui_key(ImGuiKey::DownArrow, 4);
        imgui.set_imgui_key(ImGuiKey::PageUp, 5);
        imgui.set_imgui_key(ImGuiKey::PageDown, 6);
        imgui.set_imgui_key(ImGuiKey::Home, 7);
        imgui.set_imgui_key(ImGuiKey::End, 8);
        imgui.set_imgui_key(ImGuiKey::Delete, 9);
        imgui.set_imgui_key(ImGuiKey::Backspace, 10);
        imgui.set_imgui_key(ImGuiKey::Enter, 11);
        imgui.set_imgui_key(ImGuiKey::Escape, 12);

        {
            let style = imgui.style_mut();
            style.anti_aliased_lines = false;
            style.anti_aliased_shapes = false;
        }

        Context {
            imgui: imgui,
            mouse_pos: (0, 0),
            mouse_pressed: (false, false, false),
            mouse_wheel: 0.0,
        }
    }

    pub fn update(&mut self, event: glutin::Event) {
        use glium::glutin::ElementState::Pressed;
        use glium::glutin::Event::*;
        use glium::glutin::{MouseButton, MouseScrollDelta};
        use glium::glutin::VirtualKeyCode as Key;

        match event {
            KeyboardInput(state, _, Some(code)) => {
                let pressed = state == Pressed;
                match code {
                    Key::Tab => self.imgui.set_key(0, pressed),
                    Key::Left => self.imgui.set_key(1, pressed),
                    Key::Right => self.imgui.set_key(2, pressed),
                    Key::Up => self.imgui.set_key(3, pressed),
                    Key::Down => self.imgui.set_key(4, pressed),
                    Key::PageUp => self.imgui.set_key(5, pressed),
                    Key::PageDown => self.imgui.set_key(6, pressed),
                    Key::Home => self.imgui.set_key(7, pressed),
                    Key::End => self.imgui.set_key(8, pressed),
                    Key::Delete => self.imgui.set_key(9, pressed),
                    Key::Back => self.imgui.set_key(10, pressed),
                    Key::Return => self.imgui.set_key(11, pressed),
                    Key::Escape => self.imgui.set_key(12, pressed),
                    Key::LControl | Key::RControl => self.imgui.set_key_ctrl(pressed),
                    Key::LShift | Key::RShift => self.imgui.set_key_shift(pressed),
                    Key::LAlt | Key::RAlt => self.imgui.set_key_alt(pressed),
                    _ => {},
                }
            },
            MouseMoved(x, y) => self.mouse_pos = (x, y),
            MouseInput(state, MouseButton::Left) => self.mouse_pressed.0 = state == Pressed,
            MouseInput(state, MouseButton::Right) => self.mouse_pressed.1 = state == Pressed,
            MouseInput(state, MouseButton::Middle) => self.mouse_pressed.2 = state == Pressed,
            MouseWheel(MouseScrollDelta::LineDelta(_, y), _) |
            MouseWheel(MouseScrollDelta::PixelDelta(_, y), _) => self.mouse_wheel = y,
            ReceivedCharacter(c) => self.imgui.add_input_character(c),
            _ => {},
        }
    }

    pub fn frame(&mut self, metrics: FrameMetrics) -> Ui {
        let scale = self.imgui.display_framebuffer_scale();
        self.imgui
            .set_mouse_pos(self.mouse_pos.0 as f32 / scale.0,
                           self.mouse_pos.1 as f32 / scale.1);
        self.imgui
            .set_mouse_down(&[self.mouse_pressed.0,
                              self.mouse_pressed.1,
                              self.mouse_pressed.2,
                              false,
                              false]);
        self.imgui.set_mouse_wheel(self.mouse_wheel / scale.1);
        self.mouse_wheel = 0.0;

        let FrameMetrics {
            size_pixels,
            size_points,
            delta_time,
        } = metrics;

        self.imgui
            .frame((size_points.width, size_points.height),
                   (size_pixels.width, size_pixels.height),
                   delta_time)
    }
}

pub fn checkbox(ui: &Ui, text: imgui::ImStr, initial_value: bool) -> Option<bool> {
    let mut value = initial_value;
    ui.checkbox(text, &mut value);

    if value != initial_value {
        Some(value)
    } else {
        None
    }
}

pub fn slider_float(ui: &Ui,
                    text: imgui::ImStr,
                    initial_value: f32,
                    min: f32,
                    max: f32)
                    -> Option<f32> {
    use std::f32;

    let mut value = initial_value;
    ui.slider_float(text, &mut value, min, max).build();

    if f32::abs(value - initial_value) > f32::EPSILON {
        Some(value)
    } else {
        None
    }
}

pub fn slider_int(ui: &Ui,
                  text: imgui::ImStr,
                  initial_value: i32,
                  min: i32,
                  max: i32)
                  -> Option<i32> {
    let mut value = initial_value;
    ui.slider_int(text, &mut value, min, max).build();

    if value != initial_value {
        Some(value)
    } else {
        None
    }
}
