use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

pub fn midpoint_arc(radius: f32, p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    set_radius(Point3::midpoint(p0, p1), radius)
}

pub fn face_normal(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Vector3<f32> {
    let cross = Vector3::cross(p1 - p0, p2 - p0);
    cross / cross.magnitude()
}

pub fn set_radius(point: Point3<f32>, radius: f32) -> Point3<f32> {
    Point3::from_vec(point.to_vec().normalize_to(radius))
}

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
