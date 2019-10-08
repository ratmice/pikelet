use cgmath::prelude::*;
use cgmath::{PerspectiveFov, Point3, Rad, Vector3};
use engine::camera::{Camera, ComputedCamera};
use geomath::{GeoPoint, GeoVector, GreatCircle};

#[derive(Clone, Debug)]
pub struct TurntableCamera {
    pub rotation: Rad<f32>,
    pub rotation_delta: Rad<f32>,
    pub zoom_delta: f32,
    pub xz_radius: f32,
    pub y_height: f32,
    pub near: f32,
    pub far: f32,
}

impl TurntableCamera {
    pub fn update(&mut self, delta_time: f32) {
        self.rotation += self.rotation_delta * delta_time;
        self.xz_radius += self.zoom_delta * delta_time;
    }

    pub fn reset_motion(&mut self) {
        self.rotation_delta = Rad(0.0);
        self.zoom_delta = 0.0;
    }

    pub fn compute(&self, aspect: f32) -> ComputedCamera {
        let camera = Camera {
            up: Vector3::unit_y(),
            position: Point3 {
                x: Rad::sin(self.rotation) * self.xz_radius,
                y: self.y_height,
                z: Rad::cos(self.rotation) * self.xz_radius,
            },
            target: Point3::origin(),
            projection: PerspectiveFov {
                aspect,
                fovy: Rad::full_turn() / 6.0,
                near: self.near,
                far: self.far,
            },
        };

        camera.compute()
    }
}

#[derive(Clone, Debug)]
pub struct FirstPersonCamera {
    pub location: GeoPoint<f32>,
    pub direction: GeoVector<f32>,
    // pub inclination: Rad<f32>,
    pub speed: f32,
    pub radius: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl FirstPersonCamera {
    pub fn update(&mut self, delta_time: f32) {
        self.location = self.location + (self.direction * self.speed * delta_time);
    }

    pub fn reset_motion(&mut self) {
        self.speed = 0.0;
    }

    pub fn compute(&self, aspect: f32) -> ComputedCamera {
        let position = self.location.to_point(self.radius + self.height);
        let great_circle = GreatCircle::from_point_vector(self.location, self.direction);
        let target = position + Vector3::cross(great_circle.normal(), self.location.up());

        let camera = Camera {
            up: self.location.up(),
            position,
            target,
            projection: PerspectiveFov {
                aspect,
                fovy: Rad::full_turn() / 6.0,
                near: self.near,
                far: self.far,
            },
        };

        camera.compute()
    }
}