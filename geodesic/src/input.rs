use cgmath::Point2;
use glium::glutin;

pub enum Event {
    CloseApp,
    SetShowingMesh(bool),
    SetShowingStarField(bool),
    SetWireframe(bool),
    ToggleUi,
    ResetState,
    DragStart,
    DragEnd,
    ZoomStart,
    ZoomEnd,
    MousePosition(Point2<i32>),
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
            MouseMoved((x, y)) => Event::MousePosition(Point2::new(x, y)),
            _ => Event::NoOp,
        }
    }
}
