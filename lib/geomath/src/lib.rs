extern crate cgmath;
extern crate rand;

use cgmath::prelude::*;
use cgmath::BaseFloat;
use cgmath::{Point3, Rad, Vector3};
use rand::{Rand, Rng};
use rand::distributions::range::SampleRange;
use std::ops::*;

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
            lat: A::atan(src.up.y / src.up.x),
            // Should not need `A::acos(src.up.z / src.up.magnitude())` because
            // `src.0` is a unit vector, barring rounding errors
            long: A::acos(src.up.z),
        }
    }
}

impl<A: Angle> Rand for LatLong<A>
    where A::Unitless: BaseFloat + Rand + SampleRange
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
        GeoPoint {
            up: Vector3 {
                x: sin_lat * A::cos(src.long),
                y: sin_lat * A::sin(src.long),
                z: A::cos(src.lat),
            },
        }
    }
}

pub fn arc_length<T: BaseFloat>(angle: Rad<T>, radius: T) -> Rad<T> {
    angle * radius
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
pub struct GeoPoint<T> {
    /// The normalized up vector with respect to the origin
    up: Vector3<T>,
}

impl<T: BaseFloat> GeoPoint<T> {
    pub fn north() -> GeoPoint<T> {
        GeoPoint { up: Vector3::unit_x() }
    }

    pub fn south() -> GeoPoint<T> {
        GeoPoint::north().antipode()
    }

    pub fn up(self) -> Vector3<T> {
        self.up
    }

    pub fn midpoint(self, other: GeoPoint<T>) -> GeoPoint<T> {
        GeoPoint { up: Vector3::normalize(self.up + other.up) }
    }

    pub fn antipode(self) -> GeoPoint<T> {
        GeoPoint { up: -self.up }
    }

    pub fn distance(self, other: GeoPoint<T>) -> Rad<T> {
        Vector3::angle(self.up, other.up)
    }

    pub fn to_point(self, radius: T) -> Point3<T> {
        Point3::from_vec(self.up) * radius
    }
}

impl<T: BaseFloat> Rand for GeoPoint<T>
    where T: Rand + SampleRange
{
    fn rand<R: Rng>(rng: &mut R) -> GeoPoint<T> {
        GeoPoint::from(LatLong::<Rad<T>>::rand(rng))
    }
}

/// A tangent vector on the unit sphere
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GeoVector<T> {
    forward: Vector3<T>,
}

impl<T: BaseFloat> GeoVector<T> {
    pub fn forward(self) -> Vector3<T> {
        self.forward
    }
}

impl<T: BaseFloat> Add for GeoVector<T> {
    type Output = GeoVector<T>;

    fn add(self, other: GeoVector<T>) -> GeoVector<T> {
        GeoVector { forward: self.forward + other.forward }
    }
}

impl<T: BaseFloat> Sub for GeoVector<T> {
    type Output = GeoVector<T>;

    fn sub(self, other: GeoVector<T>) -> GeoVector<T> {
        GeoVector { forward: self.forward - other.forward }
    }
}

impl<T: BaseFloat> Neg for GeoVector<T> {
    type Output = GeoVector<T>;

    fn neg(self) -> GeoVector<T> {
        GeoVector { forward: -self.forward }
    }
}

impl<T: BaseFloat> Mul<T> for GeoVector<T> {
    type Output = GeoVector<T>;

    fn mul(self, other: T) -> GeoVector<T> {
        GeoVector { forward: self.forward * other }
    }
}

impl<T: BaseFloat> Div<T> for GeoVector<T> {
    type Output = GeoVector<T>;

    fn div(self, other: T) -> GeoVector<T> {
        GeoVector { forward: self.forward / other }
    }
}

impl<T: BaseFloat> Rem<T> for GeoVector<T> {
    type Output = GeoVector<T>;

    fn rem(self, other: T) -> GeoVector<T> {
        GeoVector { forward: self.forward % other }
    }
}

impl<T: BaseFloat> Zero for GeoVector<T> {
    fn is_zero(&self) -> bool {
        self.forward.is_zero()
    }

    fn zero() -> GeoVector<T> {
        GeoVector { forward: Vector3::zero() }
    }
}

impl<T: BaseFloat> VectorSpace for GeoVector<T> {
    type Scalar = T;
}

/// A great circle on a sphere.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GreatCircle<T> {
    /// The normal vector of the great-circle plane.
    normal: Vector3<T>,
}

impl<T: BaseFloat> GreatCircle<T> {
    /// Construct a great-circle from two points on a sphere. Note that this
    /// will result in an invalid value if the points are on opposite sides
    /// of the sphere.
    pub fn from_points(a: GeoPoint<T>, b: GeoPoint<T>) -> GreatCircle<T> {
        GreatCircle { normal: Vector3::cross(a.up, b.up).normalize() }
    }

    /// The normal vector of the great-circle plane.
    pub fn normal(self) -> Vector3<T> {
        self.normal
    }
}
