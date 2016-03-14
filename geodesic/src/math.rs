use cgmath::{Angle, One, Zero};
use cgmath::Matrix4;
use cgmath::{Point3, Point};
use cgmath::{Vector3, Vector, EuclideanVector};
use rand::{Rand, Rng};
use rand::distributions::range::SampleRange;

pub fn midpoint(p0: &Point3<f32>, p1: &Point3<f32>) -> Point3<f32> {
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

pub fn array_p3(p: Point3<f32>) -> [f32; 3] {
    p.into()
}

pub fn array_m4(m: Matrix4<f32>) -> [[f32; 4]; 4] {
    m.into()
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct Polar<A: Angle> {
    pub radius: A::Unitless,
    pub inclination: A,
    pub azimuth: A,
}

impl<A: Angle> Polar<A> {
    pub fn new(radius: A::Unitless, inclination: A, azimuth: A) -> Polar<A> {
        Polar {
            radius: radius,
            inclination: inclination,
            azimuth: azimuth,
        }
    }

    pub fn rand_spherical<R: Rng>(rng: &mut R, radius: A::Unitless) -> Polar<A> where
        A::Unitless: Rand + SampleRange,
    {
        // From http://mathworld.wolfram.com/SpherePointPicking.html

        let u: A::Unitless = rng.gen_range(A::Unitless::zero(), A::Unitless::one());
        let v: A::Unitless = rng.gen_range(A::Unitless::zero(), A::Unitless::one());

        Polar::new(
            radius,
            A::full_turn() * u,
            A::acos((v + v) - A::Unitless::one())
        )
    }
}

impl<A: Angle> From<Point3<A::Unitless>> for Polar<A> {
    fn from(src: Point3<A::Unitless>) -> Polar<A> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates

        let radius = src.to_vec().length();

        Polar::new(
            src.to_vec().length(),
            A::acos(src.z / radius),
            A::atan(src.y / src.x),
        )
    }
}

impl<A: Angle> Into<Point3<A::Unitless>> for Polar<A> {
    fn into(self) -> Point3<A::Unitless> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates

        let sin_inclination = A::sin(self.inclination);

        Point3::new(
            self.radius * sin_inclination * A::cos(self.azimuth),
            self.radius * sin_inclination * A::sin(self.azimuth),
            self.radius * A::cos(self.inclination),
        )
    }
}
