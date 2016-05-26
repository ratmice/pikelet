#![allow(dead_code, unused_imports, unused_variables)]

use glium::Display;
use glium::{IndexBuffer, Program, VertexBuffer};
use glium::glutin::Event as GlutinEvent;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver};

#[derive(Copy, Clone, Debug, PartialEq)]
struct FrameData {
    delta_time: f32,
    frames_per_second: f32,
    window_dimensions: (u32, u32),
    hidpi_factor: f32,
}

enum UpdateEvent {
    WindowEvent(GlutinEvent),
    FrameRequested(FrameData),
}

enum RenderEvent {
    Data(RenderData),
    Quit,
}

struct RenderData;

struct Game {
    update_rx: Receiver<UpdateEvent>,
    render_tx: Sender<RenderEvent>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Loop {
    Continue,
    Break,
}

impl Game {
    fn new(update_rx: Receiver<UpdateEvent>, render_tx: Sender<RenderEvent>) -> Game {
        Game {
            update_rx: update_rx,
            render_tx: render_tx,
        }
    }

    fn handle_user_input(&mut self, event: GlutinEvent) -> Loop {
        use glium::glutin::ElementState::*;
        use glium::glutin::Event::*;
        use glium::glutin::MouseButton;
        use glium::glutin::VirtualKeyCode as Key;

        match event {
            Closed | KeyboardInput(Pressed, _, Some(Key::Escape)) => return Loop::Break,
            _ => {},
        }

        Loop::Continue
    }

    fn handle_frame_update(&mut self, frame_data: FrameData) -> Loop {
        unimplemented!()
    }

    fn send_render_data(&self, frame_data: FrameData) {
        // build render data and move to render thread
        self.render_tx.send(RenderEvent::Data(RenderData)).unwrap();
    }

    fn send_quit(&self) {
        self.render_tx.send(RenderEvent::Quit).unwrap();
    }

    fn update(&mut self, event: UpdateEvent) -> Loop {
        match event {
            UpdateEvent::WindowEvent(event) => self.handle_user_input(event),
            UpdateEvent::FrameRequested(frame_data) => {
                self.send_render_data(frame_data);
                self.handle_frame_update(frame_data)
            },
        }
    }

    fn run(mut self) {
        loop {
            let event = self.update_rx.recv().unwrap();

            if self.update(event) == Loop::Break {
                self.send_quit();
                break;
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}

struct Renderer<'a> {
    display: &'a Display,

    programs: HashMap<String, (Box<Path>, Program)>,
    vertex_buffers: HashMap<String, VertexBuffer<Vertex>>,
}

impl<'a> Renderer<'a> {
    fn new(display: &Display) -> Renderer {
        Renderer {
            display: display,

            programs: HashMap::new(),
            vertex_buffers: HashMap::new(),
        }
    }

    fn update(&mut self, data: RenderData) {
        // Update buffers etc...
        unimplemented!()
    }

    fn draw(&self) {
        unimplemented!()
    }

    fn frame_data(&self) -> FrameData {
        let window = self.display.get_window().unwrap();

        FrameData {
            delta_time: 0.0,
            frames_per_second: 0.0,
            window_dimensions: window.get_inner_size_points().unwrap(),
            hidpi_factor: window.hidpi_factor(),
        }
    }
}

pub fn run() {
    use find_folder::Search as FolderSearch;
    use glium::DisplayBuild;
    use glium::glutin::WindowBuilder;
    use notify::{RecommendedWatcher, Watcher};
    use std::thread;
    use std::sync::mpsc;

    // Create channels
    let (update_tx, update_rx) = mpsc::channel();
    let (render_tx, render_rx) = mpsc::channel();
    let (watcher_tx, watcher_rx) = mpsc::channel();

    // Spawn file watcher (rsnotify)
    let resources_path = FolderSearch::ParentsThenKids(3, 3).for_folder("resources").unwrap();
    let mut watcher = RecommendedWatcher::new(watcher_tx).unwrap();
    watcher.watch(&resources_path).unwrap();

    // Spawn update thread
    thread::spawn(|| {
        // Spawn worker threads for asynchronous things like procedural generation
        // tasks etc. Could be a thread pool?
        // thread::spawn(|| {});

        Game::new(update_rx, render_tx).run();
    });

    let display = WindowBuilder::new()
        .with_title("Voyager")
        .with_dimensions(1000, 500)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let mut renderer = Renderer::new(&display);

    'main: loop {
        // Collect user input, and send to update thread
        for event in display.poll_events() {
            update_tx.send(UpdateEvent::WindowEvent(event)).unwrap();
        }

        // Check for file updates
        while let Ok(event) = watcher_rx.try_recv() {
            // TODO: Recompile shaders, etc..
            println!("{:?}", event);
        }

        // Get latest data from update thread
        while let Ok(event) = render_rx.try_recv() {
            match event {
                RenderEvent::Data(data) => renderer.update(data),
                RenderEvent::Quit => break 'main,
            }
        }

        renderer.draw();

        // Send latest frame data to update thread
        update_tx.send(UpdateEvent::FrameRequested(renderer.frame_data())).unwrap();
    }
}
