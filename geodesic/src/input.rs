use cgmath::Point2;
use glium::glutin;

pub enum Event {
    Tick { window_dimensions: (u32, u32), hidpi_factor: f32, delta_time: f32 },
    CloseApp,
    SetShowingStarField(bool),
    SetUiCapturingMouse(bool),
    SetWireframe(bool),
    ToggleUi,
    ResetState,
    DragStart,
    DragEnd,
    ZoomStart,
    ZoomEnd,
    MousePosition(Point2<i32>),
    UpdatePlanetSubdivisions(usize),
    NoOp,
}

impl From<glutin::Event> for Event {
    fn from(src: glutin::Event) -> Event {
        use glium::glutin::ElementState::*;
        use glium::glutin::Event::*;
        use glium::glutin::MouseButton;
        use glium::glutin::VirtualKeyCode as Key;

        match src {
            Closed | KeyboardInput(Pressed, _, Some(Key::Escape)) => Event::CloseApp,
            KeyboardInput(Pressed, _, Some(Key::R)) => Event::ResetState,
            KeyboardInput(Pressed, _, Some(Key::U)) => Event::ToggleUi,
            MouseInput(Pressed, MouseButton::Left) => Event::DragStart,
            MouseInput(Released, MouseButton::Left) => Event::DragEnd,
            MouseInput(Pressed, MouseButton::Right) => Event::ZoomStart,
            MouseInput(Released, MouseButton::Right) => Event::ZoomEnd,
            MouseMoved(x, y) => Event::MousePosition(Point2::new(x, y)),
            _ => Event::NoOp,
        }
    }
}
