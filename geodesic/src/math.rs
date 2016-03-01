use cgmath::{Point3, Point};
use cgmath::{Vector3, EuclideanVector};

pub fn midpoint(p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    Point3::from_vec(p0.to_vec() + p1.to_vec()) * 0.5
}

pub fn face_normal(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Vector3<f32> {
    let cross = (p1 - p0).cross(p2 - p0);
    cross / cross.length()
}

pub fn project_to_radius(point: Point3<f32>, radius: f32) -> Point3<f32> {
    Point3::from_vec(point.to_vec().normalize_to(radius))
}
