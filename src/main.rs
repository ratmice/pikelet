//!
//! Voyager
//!

mod system;

use std::cell::RefCell;
use std::env;
use std::io::Error as IoError;

use env_logger;
use log;

use winit::error::OsError;
use winit::event::{self, ElementState, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

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
                }
            }
        };
        log::trace!("Requesting wgpu device and queue");
        let (device, queue) = {
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
        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: size.width.round() as u32,
                height: size.height.round() as u32,
                present_mode: wgpu::PresentMode::Vsync,
            },
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

    #[allow(unused)]
    pub fn window_size(&self) -> (u32, u32) {
        self.window
            .inner_size()
            .to_physical(self.window.hidpi_factor())
            .into()
    }
}

fn main() -> VoyagerResult<()> {
    if env::var_os(VOYAGER_LOG).is_none() {
        std::env::set_var(VOYAGER_LOG, "warn,voyager=debug");
    }
    env_logger::init_from_env(VOYAGER_LOG);
    log::info!("Voyager startup.");

    let mut app = AppContext::initialize()?;

    log::debug!("Initializing renderer.");
    let renderer = rendering::Renderer::initialize(&mut app.device)?;

    // TODO initialize any other systems (audio, scripts, ...)

    log::debug!("Entering main loop.");
    app.event_loop.run(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                event::WindowEvent::KeyboardInput { input, .. } => match input {
                    event::KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        modifiers,
                        ..
                    } => match keycode {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::W => {
                            if modifiers.alt {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            },
            event::Event::DeviceEvent { event, .. } => match event {
                event::DeviceEvent::MouseMotion {
                    delta: (delta_x, delta_y),
                } => {
                    log::trace!("Mouse delta: {}, {}", delta_x, delta_y);
                }
                _ => (),
            },
            _ => (),
        }
    });

    log::info!("Voyager shutdown.");
    Ok(())
}
