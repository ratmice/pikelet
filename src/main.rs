//!
//! Voyager
//!

mod system;

use std::env;
use std::io::{Error as IoError};
use std::cell::RefCell;

use log;
use env_logger;

use winit::event;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::error::OsError;

use system::rendering;


#[derive(Clone, Debug)]
#[allow(unused)]
pub enum VoyagerError {
    Runtime(String),
    Config(String),
}

impl From<OsError> for VoyagerError {
    fn from(err: OsError) -> Self {
        VoyagerError::Runtime(err.to_string())
    }
}

impl From<IoError> for VoyagerError {
    fn from(err: IoError) -> Self {
        VoyagerError::Runtime(format!("IO Error: {}", err.to_string()))
    }
}

pub type VoyagerResult<T> = Result<T, VoyagerError>;

const VOYAGER_LOG: &str = "VOYAGER_LOG";


struct AppContext {
    event_loop: EventLoop<()>,
    window: Window,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: RefCell<wgpu::Queue>,
    swap_chain: RefCell<wgpu::SwapChain>,
}

impl AppContext {
    pub fn initialize() -> VoyagerResult<Self> {
        log::trace!("Creating window system event loop");
        let event_loop = EventLoop::new();
        log::trace!("Creating window");
        let window = Window::new(&event_loop)?;
        log::trace!("Creating wgpu surface");
        let surface = wgpu::Surface::create(&window);
        log::trace!("Requesting wgpu adapter");;
        let adapter = {
            let req = wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                backends: wgpu::BackendBit::PRIMARY,
            };
            match wgpu::Adapter::request(&req) {
                Some(adapter) => adapter,
                None => {
                    return Err(VoyagerError::Runtime("No available adapter.".to_string()));
                },
            }
        };
        log::trace!("Requesting wgpu device and queue");
        let (device, mut queue) = {
            let desc = wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            };
            adapter.request_device(&desc)
        };
        log::trace!("Creating swap chain");
        let size = window.inner_size().to_physical(window.hidpi_factor());
        let mut swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: size.width.round() as u32,
                height: size.height.round() as u32,
                present_mode: wgpu::PresentMode::Vsync,
            }
        );

        Ok(AppContext {
            event_loop,
            window,
            surface,
            adapter,
            device,
            queue: RefCell::new(queue),
            swap_chain: RefCell::new(swap_chain),
        })
    }

    pub fn window_size(&self) -> (u32, u32) {
        self.window.inner_size()
            .to_physical(self.window.hidpi_factor())
            .into()
    }
}


fn main() -> VoyagerResult<()> {
    if let None = env::var_os(VOYAGER_LOG) {
        std::env::set_var(VOYAGER_LOG, "warn,voyager=debug");
    }
    env_logger::init_from_env(VOYAGER_LOG);
    log::info!("Voyager startup.");

    let mut app = AppContext::initialize()?;

    log::debug!("Initializing renderer.");
    let renderer = rendering::Renderer::initialize(&mut app.device)?;

    log::info!("Voyager shutdown.");
    Ok(())
}
