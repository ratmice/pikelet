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
