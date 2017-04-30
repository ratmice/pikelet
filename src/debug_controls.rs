use imgui::{self, Ui};

use InputEvent;
use engine::ui;

pub struct DebugControls {
    pub is_wireframe: bool,
    pub is_showing_star_field: bool,
    pub is_limiting_fps: bool,
    pub is_ui_capturing_mouse: bool,
    pub planet_subdivs: i32,
    pub planet_radius: f32,
    pub star_field_radius: f32,
}

impl DebugControls {
    pub fn render(&self, ui: &Ui) -> Vec<InputEvent> {
        use InputEvent::*;

        let mut events = Vec::new();

        ui.window(im_str!("State"))
            .position((10.0, 10.0), imgui::ImGuiSetCond_FirstUseEver)
            .size((300.0, 250.0), imgui::ImGuiSetCond_FirstUseEver)
            .build(|| {
                ui::checkbox(ui, im_str!("Wireframe"), self.is_wireframe)
                    .map(|v| events.push(SetWireframe(v)));
                ui::checkbox(ui, im_str!("Show star field"), self.is_showing_star_field)
                    .map(|v| events.push(SetShowingStarField(v)));
                ui::checkbox(ui, im_str!("Limit FPS"), self.is_limiting_fps)
                    .map(|v| events.push(SetLimitingFps(v)));
                ui::slider_int(ui,
                               im_str!("Planet subdivisions"),
                               self.planet_subdivs,
                               1,
                               8)
                        .map(|v| events.push(SetPlanetSubdivisions(v as usize)));
                ui::slider_float(ui, im_str!("Planet radius"), self.planet_radius, 0.0, 2.0)
                    .map(|v| events.push(SetPlanetRadius(v)));
                ui::slider_float(ui,
                                 im_str!("Star field radius"),
                                 self.star_field_radius,
                                 0.0,
                                 20.0)
                        .map(|v| events.push(SetStarFieldRadius(v)));

                if ui.small_button(im_str!("Reset state")) {
                    events.push(ResetState);
                }
            });

        if ui.want_capture_mouse() != self.is_ui_capturing_mouse {
            events.push(SetUiCapturingMouse(ui.want_capture_mouse()));
        }

        events
    }
}
