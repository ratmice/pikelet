#![feature(globs)]

extern crate native;

use platform::Platform;
use resources::ResourceManager;
use graphics::GraphicsManager;
use game::Game;

mod platform;
mod resources;
mod graphics;
mod game;

struct Application {
    target_delta: f64,
    pub game_time: f64,
    pub resources: ResourceManager,
    pub platform: platform::glfw::GlfwPlatform,
    graphics: GraphicsManager,
    // animations: AnimationManager,
    // physics: PhysicsManager,
    // fx: EffectsManager,
    // audio: AudioManager,
    // game: VoyagerGame,
}

impl Application {
    fn init() -> Application {
        let resource_manager = ResourceManager::init();
        let platform = platform::glfw::init();
        let graphics_manager = GraphicsManager::init(&platform);
        //AnimationManager::init()
        //PhysicsManager::init()
        //EffectsManager::init()
        //AudioManager::init()
        //let game = game::voyager_game::init();


        Application {
            target_delta: 1.0 / 120.0,
            game_time: 0.0,
            platform: platform,
            resources: resource_manager,
            graphics: graphics_manager
        }
    }
    
    fn render(&self) {
        self.graphics.clear();

        self.platform.swap();
    }

    fn update(&self, delta:f64) {
        // update animation

        // update physics

        // update fx

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
        }

        //self.game.shutdown();
        //self.audio.shutdown();
        //self.animation.shutdown();
        //self.fx.shutdown();
        //self.physics.shutdown();
        //self.graphics.shutdown();
        self.platform.shutdown();
        self.resources.shutdown();
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
