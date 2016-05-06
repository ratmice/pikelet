use cgmath::prelude::*;
use cgmath::{One, Zero, Point3, Vector3};
use num_traits::{Float, cast};
use rand::{Rand, Rng};
use rand::distributions::range::SampleRange;

pub fn midpoint(p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    Point3::from_vec(p0.to_vec() + p1.to_vec()) * 0.5
}

pub fn midpoint_arc(radius: f32, p0: Point3<f32>, p1: Point3<f32>) -> Point3<f32> {
    set_radius(midpoint(p0, p1), radius)
}

pub fn centroid(points: &[Point3<f32>]) -> Point3<f32> {
    let sum: Vector3<f32> = points.iter()
        .map(|p| p.to_vec())
        .fold(Vector3::zero(), |acc, v| acc + v);

    Point3::from_vec(sum / points.len() as f32)
}

pub fn face_normal(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Vector3<f32> {
    let cross = Vector3::cross(p1 - p0, p2 - p0);
    cross / cross.magnitude()
}

pub fn set_radius(point: Point3<f32>, radius: f32) -> Point3<f32> {
    Point3::from_vec(point.to_vec().normalize_to(radius))
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
            A::acos((v + v) - A::Unitless::one()),
        )
    }
}

impl<A: Angle> From<Point3<A::Unitless>> for Polar<A> {
    fn from(src: Point3<A::Unitless>) -> Polar<A> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates

        let radius = src.to_vec().magnitude();

        Polar::new(
            src.to_vec().magnitude(),
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

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct LatLong<A: Angle> {
    pub lat: A,
    pub long: A,
}

impl<A: Angle> LatLong<A> {
    pub fn new(lat: A, long: A) -> LatLong<A> {
        LatLong { lat: lat, long: long }
    }

    pub fn great_circle_distance(self, other: LatLong<A>) -> A::Unitless {
        // http://www.movable-type.co.uk/scripts/latlong.html
        // http://www.fssnip.net/4a

        fn hav<A: Angle>(a: A) -> A::Unitless {
            let two: A::Unitless = cast(2.0).unwrap();
            let tmp = A::sin(a / two);
            tmp * tmp
        }

        let one = A::Unitless::one();
        let two: A::Unitless = cast(2.0).unwrap();

        let dlat = other.lat - self.lat;
        let dlong = other.long - self.long;

        let a = hav(dlat) + A::cos(self.lat) * A::cos(other.lat) * hav(dlong);

        // Naughty!
        Float::atan2(Float::sqrt(a), Float::sqrt(one - a)) * two
    }

    pub fn bearing(self, other: Self) -> A {
        // http://williams.best.vwh.net/avform.htm#Crs
        // http://mathforum.org/library/drmath/view/55417.html

        let dlong = other.long - self.long;
        let x = A::sin(dlong) * A::cos(other.lat);
        let y = A::cos(self.lat) * A::sin(other.lat)
              - A::sin(self.lat) * A::cos(other.lat) * A::cos(dlong);

        A::atan2(x, y) % A::full_turn()
    }
}

impl<A: Angle> From<Vector3<A::Unitless>> for LatLong<A> {
    fn from(src: Vector3<A::Unitless>) -> LatLong<A> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates

        LatLong {
            lat: A::atan(src.y / src.x),
            long: A::acos(src.z / src.magnitude()),
        }
    }
}

impl<A: Angle> Into<Vector3<A::Unitless>> for LatLong<A> {
    fn into(self) -> Vector3<A::Unitless> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
        let sin_lat = A::sin(self.lat);
        Vector3 {
            x: sin_lat * A::cos(self.long),
            y: sin_lat * A::sin(self.long),
            z: A::cos(self.lat),
        }
    }
}

impl<A: Angle> Rand for LatLong<A> where
    A::Unitless: Rand + SampleRange,
{
    fn rand<R: Rng>(rng: &mut R) -> LatLong<A> {
        // From http://mathworld.wolfram.com/SpherePointPicking.html

        let u: A::Unitless = rng.gen_range(A::Unitless::zero(), A::Unitless::one());
        let v: A::Unitless = rng.gen_range(A::Unitless::zero(), A::Unitless::one());

        LatLong {
            lat: A::acos((v + v) - A::Unitless::one()),
            long: A::full_turn() * u,
        }
    }
}
