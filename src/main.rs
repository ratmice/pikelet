#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(doc_markdown))]
#![cfg_attr(feature = "clippy", allow(new_without_default))]

extern crate cgmath;
#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
extern crate find_folder;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
#[macro_use]
extern crate maplit;
extern crate notify;
extern crate num_traits;
#[macro_use]
extern crate quick_error;
extern crate rand;
extern crate rayon;
extern crate time;
#[macro_use]
extern crate itertools;

extern crate job_queue;

use cgmath::Vector2;
use glium::Frame;
use std::sync::mpsc::Sender;
use std::time::Duration;

use game::{InputEvent, State};
use math::Size2;
use render::CommandList;
use ui::Context as UiContext;

pub mod camera;
pub mod color;
mod game;
pub mod geom;
pub mod dggs;
pub mod math;
pub mod times;
mod render;
pub mod ui;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameData {
    size_points: Size2<u32>,
    size_pixels: Size2<u32>,
    delta_time: f32,
}

impl FrameData {
    fn frames_per_second(&self) -> f32 {
        match self.delta_time {
            0.0 => 0.0,
            delta_time => 1.0 / delta_time,
        }
    }

    fn framebuffer_scale(&self) -> Vector2<f32> {
        Vector2::new(
            match self.size_points.width {
                0 => 0.0,
                width => self.size_pixels.width as f32 / width as f32,
            },
            match self.size_points.height {
                0 => 0.0,
                height => self.size_pixels.height as f32 / height as f32,
            },
        )
    }
}

pub struct RenderData {
    frame_data: FrameData,
    commands: CommandList,
    state: State,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UpdateEvent {
    FrameRequested(FrameData),
    Input(InputEvent),
}

fn create_frame_data(display: &glium::Display, delta_time: f32) -> FrameData {
    let window = display.get_window().unwrap();
    let size_points = window.get_inner_size_points().unwrap();
    let size_pixels = window.get_inner_size_pixels().unwrap();

    FrameData {
        size_points: Size2::new(size_points.0, size_points.1),
        size_pixels: Size2::new(size_pixels.0, size_pixels.1),
        delta_time: delta_time,
    }
}

fn render_ui(target: &mut Frame, ui_context: &mut UiContext, frame_data: FrameData, state: &State, update_tx: &Sender<UpdateEvent>) {
    ui_context.render(target, frame_data, |ui| {
        game::run_ui(ui, frame_data, state, |event| {
            // FIXME: could cause a panic on the slim chance that the update thread
            //  closes during ui rendering.
            update_tx.send(UpdateEvent::Input(event)).unwrap();
        });
    }).unwrap();
}

macro_rules! try_or {
    ($e:expr, $or:expr) => {
        match $e { Ok(x) => x, Err(_) => $or }
    };
}

fn main() {
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    use std::sync::mpsc;
    use std::thread;

    let display =
        WindowBuilder::new()
            .with_title("Voyager!")
            .with_dimensions(1000, 500)
            .with_depth_buffer(24)
            .build_glium()
            .unwrap();

    let frame_data = create_frame_data(&display, 0.0);
    let mut resources = game::init_resources(&display);
    let mut ui_context = UiContext::new(&display);

    let (resource_tx, resource_rx) = mpsc::channel();
    let (render_tx, render_rx) = mpsc::sync_channel(1);

    let update_tx = game::spawn(frame_data, resource_tx, render_tx);

    'main: for time in times::in_seconds() {
        // Swap frames with update thread
        let RenderData { frame_data, commands, state } = {
            let frame_data = create_frame_data(&display, time.delta() as f32);
            let update_event = UpdateEvent::FrameRequested(frame_data);

            try_or!(update_tx.send(update_event), break 'main);
            try_or!(render_rx.recv(), break 'main)
        };
        ui_context.set_is_enabled(state.is_ui_enabled);

        // Get user input
        for event in display.poll_events() {
            ui_context.update(event.clone());
            let update_event = UpdateEvent::Input(InputEvent::from(event));

            try_or!(update_tx.send(update_event), break 'main);
        }

        // Update resources
        while let Ok(event) = resource_rx.try_recv() {
            resources.handle_event(event);
        }

        // Render frame
        let mut frame = display.draw();

        resources.draw(&mut frame, commands).unwrap();
        render_ui(&mut frame, &mut ui_context, frame_data, &state, &update_tx);

        frame.finish().unwrap();

        if state.is_limiting_fps {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
