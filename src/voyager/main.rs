#![feature(globs)]

extern crate native;

use platform::Platform;
use graphics::GraphicsManager;
use game::Game;

mod platform;
mod graphics;
mod game;

struct Application {
    target_delta: f64,
    pub game_time: f64,
    pub platform: platform::glfw::GlfwPlatform,
    // resources: ResourceManager,
    graphics: GraphicsManager,
    // animations: AnimationManager,
    // physics: PhysicsManager,
    // audio: AudioManager,
    // game: VoyagerGame,
}

impl Application {
    fn init() -> Application {
        let platform = platform::glfw::init();
        
        //ResourceManager::init()
        let graphics_manager = GraphicsManager::init(&platform);
        //AnimationManager::init()
        //PhysicsManager::init()
        //AudioManager::init()
        //let game = game::voyager_game::init();


        Application {
            target_delta: 1.0 / 120.0,
            game_time: 0.0,
            platform: platform,
            graphics: graphics_manager
        }
    }
    
    fn render(&self) {
        self.graphics.clear();
    }

    fn update(&self, delta:f64) {
        // update animation

        // update physics

        // update game

        // update audio
    }

    fn run(&mut self) {
        let mut previous_time: f64 = self.platform.get_time();

        while !self.platform.exit_requested() {
            let current_time: f64 = self.platform.get_time();
            let mut frame_time: f64 = current_time - previous_time;
            previous_time = current_time;

            // Gather input and dispatch commands
            self.platform.process_events();

            // Update
            while frame_time > 0.0 {
                let delta: f64 = self.target_delta.min(frame_time);

                self.update(delta);

                frame_time -= delta;
                self.game_time += delta;
            }

            // Draw the world
            self.render();
            self.platform.swap();
        }

        //self.game.shutdown();
        //self.audio_manager.shutdown();
        //self.physics_manager.shutdown();
        //self.animation_manager.shutdown();
        //self.graphics_manager.shutdown();
        //self.resource_manager.shutdown();
        self.platform.shutdown();
    }
}

fn main() {
    let mut app = Application::init();
    app.run()
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}
