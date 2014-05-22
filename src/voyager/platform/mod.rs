pub mod glfw;

pub trait Platform {
    fn exit_requested(&self) -> bool;
    fn process_events(&self);
    fn swap(&self);
    fn load_gl(&self, f: fn(|&str| -> Option<extern "system" fn()>));
    fn get_time(&self) -> f64;
    fn signal_shutdown(&self);
    fn shutdown(&self);
}

pub trait Command<T> {
    fn call(&self, data: T);
}

pub enum SwitchState {
    SwitchOn,
    SwitchOff,
}

pub trait InputManager {
    fn set_switch_command(&mut self, name: &str, switch: Option<Box<Command<SwitchState>>>) -> bool;
}
