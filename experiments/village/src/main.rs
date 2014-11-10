#![feature(phase)]

#[phase(plugin)]
extern crate compile_msg;

extern crate gfx;
extern crate glutin;
extern crate nalgebra;

use glutin::{KeyboardInput, Pressed, Escape, Closed};

fn main() {
    let window = glutin::Window::new().unwrap();

    unsafe { window.make_current() };

    'main: loop {
        window.swap_buffers();

        for event in window.wait_events() {
            match event {
                Closed | KeyboardInput(Pressed, _, Some(Escape)) => {
                    break 'main;
                },
                _ => {},
            }
        }
    }
}
