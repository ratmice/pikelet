use imgui::{self, Ui};

use {Event, CameraMode};
use engine::ui;

pub struct DebugControls {
    pub is_wireframe: bool,
    pub is_showing_star_field: bool,
    pub is_limiting_fps: bool,
    pub is_ui_capturing_mouse: bool,
    pub camera_mode: CameraMode,
    pub planet_subdivs: i32,
    pub planet_radius: f32,
    pub star_field_radius: f32,
}

impl DebugControls {
    pub fn render(&self, ui: &Ui) -> Vec<Event> {
        let mut events = Vec::new();

        ui.window(im_str!("State"))
            .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
            .size((300.0, 250.0), imgui::ImGuiSetCond_FirstUseEver)
            .build(|| {
                ui::checkbox(ui, im_str!("Wireframe"), self.is_wireframe)
                    .map(|v| events.push(Event::SetWireframe(v)));
                ui::checkbox(ui, im_str!("Show star field"), self.is_showing_star_field)
                    .map(|v| events.push(Event::SetShowingStarField(v)));
                ui::checkbox(ui, im_str!("Limit FPS"), self.is_limiting_fps)
                    .map(|v| events.push(Event::SetLimitingFps(v)));

                ui::combo(ui,
                          im_str!("Camera mode"),
                          match self.camera_mode {
                              CameraMode::Turntable => 0,
                              CameraMode::FirstPerson => 1,
                          },
                          &[im_str!("Turntable"), im_str!("First Person")],
                          2)
                        .map(|v| {
                            let mode = match v {
                                0 => CameraMode::Turntable,
                                1 => CameraMode::FirstPerson,
                                v => panic!("Unexpected combo index: {:?}", v),
                            };

                            events.push(Event::SetCameraMode(mode));
                        });

                ui::slider_int(ui,
                               im_str!("Planet subdivisions"),
                               self.planet_subdivs,
                               1,
                               8)
                        .map(|v| events.push(Event::SetPlanetSubdivisions(v as usize)));
                ui::slider_float(ui, im_str!("Planet radius"), self.planet_radius, 0.0, 2.0)
                    .map(|v| events.push(Event::SetPlanetRadius(v)));
                ui::slider_float(ui,
                                 im_str!("Star field radius"),
                                 self.star_field_radius,
                                 0.0,
                                 20.0)
                        .map(|v| events.push(Event::SetStarFieldRadius(v)));

                if ui.small_button(im_str!("Reset state")) {
                    events.push(Event::ResetState);
                }
            });

        if ui.want_capture_mouse() != self.is_ui_capturing_mouse {
            events.push(Event::SetUiCapturingMouse(ui.want_capture_mouse()));
        }

        events
    }
}
