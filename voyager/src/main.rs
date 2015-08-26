
extern crate glium;
extern crate vtime;

use std::thread;

use glium::{Display, DisplayBuild, Surface};
use glium::glutin;

use resources::ResourceManager;

mod resources;

struct Application {
    target_delta: f64,
    pub game_time: f64,
    display: Display,
    pub resources: ResourceManager
}

impl Application {
    fn new() -> Application {
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(1280, 720)
            .with_title(format!("Hello world"))
            .build_glium()
            .unwrap();
        
        Application {
            target_delta: 1.0 / 60.0,
            game_time: 0.0,
            display: display,
            resources: ResourceManager
        }
    }

    fn render(&self) {
        let mut frame = self.display.draw();
        frame.clear_color_and_depth((0.5, 0.5, 0.5, 1.0), 0.0);

        frame.finish();
    }

    fn update(&self, delta:f64) {
        thread::sleep_ms(5);
    }

    fn run(&mut self) {
        // startup

        // main loop
        'main: for time in vtime::seconds() {
            let mut frame_time: f64 = time.delta();

            // Gather input and dispatch commands
            for event in self.display.poll_events() {
                match event {
                    glutin::Event::Closed => break 'main,
                    _ => ()
                }
            }

            // Update
            while frame_time > 0.0 {
                let delta: f64 = self.target_delta.min(frame_time);

                self.update(delta);

                frame_time -= delta;
                self.game_time += delta;
            }

            // Draw the world
            self.render();
        }

        // shutdown
    }
}

fn main() {
    let mut app = Application::new();
    app.run()
}
