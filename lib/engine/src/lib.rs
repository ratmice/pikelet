extern crate cgmath;
#[cfg(test)]
extern crate expectest;
#[macro_use]
extern crate glium;
extern crate imgui;
#[macro_use]
extern crate quick_error;
extern crate time;

use std::time::Duration;

use math::Size2;
use render::{CommandList, ResourcesRef};

pub mod camera;
pub mod color;
pub mod input;
pub mod math;
pub mod times;
pub mod render;
pub mod ui;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Loop {
    Continue,
    Break,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameMetrics {
    pub size_points: Size2<u32>,
    pub size_pixels: Size2<u32>,
    pub delta_time: f32,
}

impl FrameMetrics {
    pub fn frames_per_second(&self) -> f32 {
        match self.delta_time {
            0.0 => 0.0,
            delta_time => 1.0 / delta_time,
        }
    }

    pub fn framebuffer_scale(&self) -> (f32, f32) {
        (match self.size_points.width {
             0 => 0.0,
             width => self.size_pixels.width as f32 / width as f32,
         },
         match self.size_points.height {
             0 => 0.0,
             height => self.size_pixels.height as f32 / height as f32,
         })
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size_pixels.width as f32 / self.size_pixels.height as f32
    }
}

pub struct RenderData<Event> {
    pub metrics: FrameMetrics,
    pub is_limiting_fps: bool,
    pub command_list: CommandList<Event>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UpdateEvent<InputEvent> {
    FrameRequested(FrameMetrics),
    Input(InputEvent),
}

fn create_frame_metrics(display: &glium::Display, delta_time: f32) -> FrameMetrics {
    let window = display.get_window().unwrap();
    let size_points = window.get_inner_size_points().unwrap();
    let size_pixels = window.get_inner_size_pixels().unwrap();

    FrameMetrics {
        size_points: Size2::new(size_points.0, size_points.1),
        size_pixels: Size2::new(size_pixels.0, size_pixels.1),
        delta_time: delta_time,
    }
}

macro_rules! try_or {
    ($e:expr, $or:expr) => {
        match $e { Ok(x) => x, Err(_) => $or }
    };
}

pub trait Application {
    type Event: Send + From<glium::glutin::Event> + 'static;

    fn init(metrics: FrameMetrics, resources: ResourcesRef) -> Self;
    fn handle_frame_request(&mut self, metrics: FrameMetrics) -> Loop;
    fn handle_input(&mut self, event: Self::Event) -> Loop;
    fn render(&self) -> RenderData<Self::Event>;
}

#[cfg_attr(feature = "cargo-clippy", allow(drop_copy))]
pub fn run<T: Application>() {
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    use std::sync::mpsc;
    use std::thread;

    use render::Renderer;

    let display = WindowBuilder::new()
        .with_title("Voyager!")
        .with_dimensions(1000, 500)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let metrics = create_frame_metrics(&display, 0.0);

    let (render_tx, render_rx) = mpsc::sync_channel(1);
    let (update_tx, update_rx) = mpsc::channel();

    let mut renderer = Renderer::new(&display);
    let resources_ref = renderer.resources().clone();

    thread::spawn(move || {
        let mut game = T::init(metrics, resources_ref);

        for event in update_rx {
            let loop_result = match event {
                UpdateEvent::FrameRequested(metrics) => {
                    // We send the data for the last frame so that the renderer
                    // can get started doing it's job in parallel!
                    render_tx
                        .send(game.render())
                        .expect("Failed to send render data");

                    game.handle_frame_request(metrics)
                },
                UpdateEvent::Input(event) => game.handle_input(event),
            };

            if loop_result == Loop::Break {
                break;
            };
        }
    });


    'main: for time in times::in_seconds() {
        // Swap frames with update thread
        let render_data = {
            let metrics = create_frame_metrics(&display, time.delta() as f32);
            let update_event = UpdateEvent::FrameRequested(metrics);

            try_or!(update_tx.send(update_event), break 'main);
            try_or!(render_rx.recv(), break 'main)
        };

        // Get user input
        for event in display.poll_events() {
            renderer.handle_ui_event(event.clone());
            let update_event = UpdateEvent::Input(event.into());
            try_or!(update_tx.send(update_event), break 'main);
        }

        // Update renderer
        renderer.poll();

        // Render frame
        let mut frame = display.draw();

        renderer
            .draw(&mut frame,
                  render_data.metrics,
                  render_data.command_list,
                  |event| drop(update_tx.send(UpdateEvent::Input(event))))
            .unwrap();

        frame.finish().unwrap();

        if render_data.is_limiting_fps {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
