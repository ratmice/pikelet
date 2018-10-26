#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Size2<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size2<T> {
    pub fn new(width: T, height: T) -> Size2<T> {
        Size2 {
            width: width,
            height: height,
        }
    }
}
