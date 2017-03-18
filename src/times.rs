use std::mem;
use std::ops::Sub;
use time;

pub struct State<T: Copy + Sub<T, Output = T>> {
    previous: T,
    current: T,
}

impl<T: Copy + Sub<T, Output = T>> State<T> {
    pub fn previous(&self) -> T {
        self.previous
    }
    pub fn current(&self) -> T {
        self.current
    }
    pub fn delta(&self) -> T {
        self.current - self.previous
    }
}

pub struct Times<T: Copy + Sub<T, Output = T>> {
    get_time: fn() -> T,
    previous: T,
}

impl<T: Copy + Sub<T, Output = T>> Iterator for Times<T> {
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

pub fn in_seconds() -> Times<f64> {
    Times {
        get_time: time::precise_time_s,
        previous: time::precise_time_s(),
    }
}

pub fn in_nanoseconds() -> Times<u64> {
    Times {
        get_time: time::precise_time_ns,
        previous: time::precise_time_ns(),
    }
}
