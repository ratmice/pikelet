#![allow(dead_code)]

extern crate time as lib;

use std::mem;
use std::ops::Sub;

pub struct State<T: Copy + Sub<T, Output = T>> {
    previous: T,
    current: T,
}

impl<T: Copy + Sub<T, Output = T>> State<T> {
    pub fn previous(&self) -> T { self.previous }
    pub fn current(&self) -> T { self.current }
    pub fn delta(&self) -> T { self.current - self.previous }
}

pub struct Deltas<T: Copy + Sub<T, Output = T>> {
    get_time: fn() -> T,
    previous: T,
}

impl<T: Copy + Sub<T, Output = T>> Iterator for Deltas<T> {
    type Item = State<T>;

    fn next(&mut self) -> Option<State<T>> {
        let current = (self.get_time)();
        let previous = mem::replace(&mut self.previous, current);
        Some(State {
            previous: previous,
            current: current,
        })
    }
}

pub fn seconds() -> Deltas<f64> {
    Deltas {
        get_time: lib::precise_time_s,
        previous: lib::precise_time_s(),
    }
}

pub fn nanoseconds() -> Deltas<u64> {
    Deltas {
        get_time: lib::precise_time_ns,
        previous: lib::precise_time_ns(),
    }
}
