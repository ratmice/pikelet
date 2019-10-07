
use crate::{VoyagerResult, VoyagerError};

use log;
use std::cell::RefCell;
//use winit::event;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::error::OsError;
use wgpu::{Surface, Adapter, Device, Queue, SwapChain};

impl From<OsError> for VoyagerError {
    fn from(err: OsError) -> Self {
        VoyagerError::Runtime(err.to_string())
    }
}


pub struct Renderer {
    window: Window,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: RefCell<Queue>,
    swap_chain: RefCell<SwapChain>,
}

impl Renderer {
    pub fn initialize(event_loop: &EventLoop<()>) -> VoyagerResult<Renderer> {
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
            match Adapter::request(&req) {
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
        Ok(Renderer {
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
