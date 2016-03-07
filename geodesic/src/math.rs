use cgmath::{Matrix4};
use cgmath::{Point3, Point};
use cgmath::{Vector3, Vector, EuclideanVector};

pub fn midpoint(p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    Point3::from_vec(p0.to_vec() + p1.to_vec()) * 0.5
}

pub fn centroid(points: &[Point3<f32>]) -> Point3<f32> {
    let sum: Vector3<f32> = points.iter()
        .map(|p| p.to_vec())
        .fold(Vector3::zero(), |acc, v| acc + v);

    Point3::from_vec(sum / points.len() as f32)
}

pub fn face_normal(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Vector3<f32> {
    let cross = (p1 - p0).cross(p2 - p0);
    cross / cross.length()
}

pub fn set_radius(point: Point3<f32>, radius: f32) -> Point3<f32> {
    Point3::from_vec(point.to_vec().normalize_to(radius))
}

pub fn array_v3(v: Vector3<f32>) -> [f32; 3] {
    v.into()
}

pub fn array_m4(m: Matrix4<f32>) -> [[f32; 4]; 4] {
    m.into()
}
