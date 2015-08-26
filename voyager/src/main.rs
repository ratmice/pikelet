
extern crate glium;

use std::thread;

use glium::glutin;

// use resources::ResourceManager;

// mod resources;

// struct Application {
//     target_delta: f64,
//     pub game_time: f64,
//     pub resources: ResourceManager
// }

// impl Application {
//     fn init() -> Application {
//         Application {
//             target_delta: 1.0 / 60.0,
//             game_time: 0.0,
//             resources: ResourceManager
//         }
//     }

//     fn render(&self) {
//     }

//     fn update(&self, delta:f64) {
//     }

//     fn run(&mut self) {
//         // TODO
//         //let mut previous_time: f64 = self.platform.get_time();
//         let mut previous_time: f64 = 0.0;

//         // TODO
//         //while !self.platform.exit_requested() {
//         let running = false;
//         while running {
//             // TODO
//             // let current_time: f64 = self.platform.get_time();
//             let current_time: f64 = 0.0;
//             let mut frame_time: f64 = current_time - previous_time;
//             previous_time = current_time;

//             // Gather input and dispatch commands

//             // Update
//             while frame_time > 0.0 {
//                 let delta: f64 = self.target_delta.min(frame_time);

//                 self.update(delta);

//                 frame_time -= delta;
//                 self.game_time += delta;
//             }

//             // Draw the world
//             self.render();
//         }

//         // shutdown
//     }
// }

fn main() {
    // let mut app = Application::init();
    // app.run()
    use glium::DisplayBuild;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title(format!("Hello world"))
        .build_glium()
        .unwrap();

    'main: loop {
        let (w, h) = display.get_framebuffer_dimensions();

        thread::sleep_ms(17);

        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => break 'main,
                _ => ()
            }
        }
    }
}
