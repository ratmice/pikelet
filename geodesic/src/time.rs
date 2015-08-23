extern crate time as lib;

use std::mem;

#[allow(dead_code)]
pub struct State {
    previous: f64,
    current: f64,
}

#[allow(dead_code)]
impl State {
    pub fn previous(&self) -> f64 { self.previous }
    pub fn current(&self) -> f64 { self.current }
    pub fn delta(&self) -> f64 { self.current - self.previous }
}

pub struct Deltas {
    previous: f64,
}

impl Iterator for Deltas {
    type Item = State;

    fn next(&mut self) -> Option<State> {
        let current = lib::precise_time_s();
        let previous = mem::replace(&mut self.previous, current);
        Some(State {
            previous: previous,
            current: current,
        })
    }
}

pub fn seconds() -> Deltas {
    Deltas { previous: lib::precise_time_s() }
}
