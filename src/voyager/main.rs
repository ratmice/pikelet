extern crate gl;
extern crate native;

use platform::Platform;

mod platform;

fn render() {
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
}

fn update(time: f64, delta:f64) {
    // update animation

    // update physics

    // update world

    // update audio
}

fn main() {
    let platform = platform::glfw::init();

    platform.load_gl(gl::load_with);

    let target_delta: f64 = 1.0 / 60.0;
    let mut game_time: f64 = 0.0;
    let mut previous_time: f64 = platform.get_time();

    while !platform.exit_requested() {
        let current_time: f64 = platform.get_time();
        let mut frame_time: f64 = current_time - previous_time;
        previous_time = current_time;

        // Gather input and dispatch commands
        platform.process_events();

        // Update
        while frame_time > 0.0 {
            let delta: f64 = target_delta.min(frame_time);

            update(game_time, delta);

            frame_time -= delta;
            game_time += delta;
        }

        // Draw the world
        render();
        platform.swap();
    }

    platform.shutdown();
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}
