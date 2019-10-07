//!
//! Voyager
//!

mod system;

use std::env;
use log;
use env_logger;
use winit::event;
use winit::event_loop::EventLoop;
use system::rendering;

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum VoyagerError {
    Runtime(String),
    Config(String),
}

pub type VoyagerResult<T> = Result<T, VoyagerError>;

const VOYAGER_LOG: &str = "VOYAGER_LOG";


fn main() -> VoyagerResult<()> {
    if let None = env::var_os(VOYAGER_LOG) {
        std::env::set_var(VOYAGER_LOG, "warn,voyager=debug");
    }
    env_logger::init_from_env(VOYAGER_LOG);
    log::info!("Voyager startup.");

    log::trace!("Creating window system event loop");
    let event_loop = EventLoop::new();

    log::debug!("Initializing renderer.");
    let renderer = rendering::Renderer::initialize(&event_loop)?;

    log::info!("Voyager shutdown.");
    Ok(())
}
