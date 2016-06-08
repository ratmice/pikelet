use cgmath::prelude::*;
use cgmath::BaseFloat;
use cgmath::{Point3, Rad, Vector3};
use rand::{Rand, Rng};
use rand::distributions::range::SampleRange;

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

/// A location on a unit sphere, described using latitude and longitude.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct LatLong<A: Angle> {
    pub lat: A,
    pub long: A,
}

impl<A: Angle> From<GeoPoint<A::Unitless>> for LatLong<A> {
    fn from(src: GeoPoint<A::Unitless>) -> LatLong<A> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
        LatLong {
            lat: A::atan(src.0.y / src.0.x),
            // Should not need `A::acos(src.0.z / src.0.magnitude())` because
            // `src.0` is a unit vector, barring rounding errors
            long: A::acos(src.0.z),
        }
    }
}

impl<A: Angle> Rand for LatLong<A> where
    A::Unitless: BaseFloat + Rand + SampleRange,
{
    fn rand<R: Rng>(rng: &mut R) -> LatLong<A> {
        // From http://mathworld.wolfram.com/SpherePointPicking.html

        let u = rng.gen_range(A::Unitless::zero(), A::Unitless::one());
        let v = rng.gen_range(A::Unitless::zero(), A::Unitless::one());

        LatLong {
            lat: A::acos((v + v) - A::Unitless::one()),
            long: A::full_turn() * u,
        }
    }
}

impl<A: Angle> From<LatLong<A>> for GeoPoint<A::Unitless> {
    fn from(src: LatLong<A>) -> GeoPoint<A::Unitless> {
        // From https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
        let sin_lat = A::sin(src.lat);
        GeoPoint(Vector3 {
            x: sin_lat * A::cos(src.long),
            y: sin_lat * A::sin(src.long),
            z: A::cos(src.lat),
        })
    }
}

pub fn arc_length<T: BaseFloat>(angle: Rad<T>, radius: T) -> T {
    angle.s * radius
}

/// A point on the surface of a sphere.
///
/// This uses an underlying vector representation to reduce the amount of
/// expensive trigonometry needed and also to avoid problems at the poles.
///
/// # References
///
/// - http://www.movable-type.co.uk/scripts/latlong-vectors.html
/// - http://www.navlab.net/Publications/A_Nonsingular_Horizontal_Position_Representation.pdf
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GeoPoint<T>(Vector3<T>);

impl<T: BaseFloat> GeoPoint<T> {
    pub fn north() -> GeoPoint<T> {
        GeoPoint(Vector3::unit_x())
    }

    pub fn south() -> GeoPoint<T> {
        GeoPoint::north().antipode()
    }

    pub fn midpoint(self, other: GeoPoint<T>) -> GeoPoint<T> {
        GeoPoint((self.0 + other.0).normalize())
    }

    pub fn antipode(self) -> GeoPoint<T> {
        GeoPoint(-self.0)
    }

    pub fn distance(self, other: GeoPoint<T>) -> Rad<T> {
        Vector3::angle(self.0, other.0)
    }

    pub fn to_point(self, radius: T) -> Point3<T> {
        Point3::from_vec(self.0) * radius
    }
}

impl<T: BaseFloat> Rand for GeoPoint<T> where
    T: Rand + SampleRange,
{
    fn rand<R: Rng>(rng: &mut R) -> GeoPoint<T> {
        GeoPoint::from(LatLong::<Rad<T>>::rand(rng))
    }
}
