
use super::Application;

pub trait Game {
    fn init(app: Application);
    fn update(delta: f64);
    fn shutdown();
}
