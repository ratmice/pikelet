use cgmath::prelude::*;
use cgmath::{PerspectiveFov, Point3, Rad, Vector3};
use engine::camera::{Camera, ComputedCamera};
use geomath::GeoPoint;

#[derive(Clone, Debug)]
pub struct TurntableCamera {
    pub rotation: Rad<f32>,
    pub rotation_delta: Rad<f32>,
    pub zoom_delta: f32,
    pub xz_radius: f32,
    pub y_height: f32,
    pub near: f32,
    pub far: f32,
    pub zoom_factor: f32,
    pub drag_factor: f32,
}

impl TurntableCamera {
    pub fn update(&mut self, delta_time: f32) {
        self.rotation += self.rotation_delta * self.drag_factor * delta_time;
        self.xz_radius += self.zoom_delta * self.zoom_factor * delta_time;
    }

    pub fn reset_motion(&mut self) {
        self.rotation_delta = Rad(0.0);
        self.zoom_delta = 0.0;
    }

    pub fn compute(&self, aspect_ratio: f32) -> ComputedCamera {
        let camera = Camera {
            up: Vector3::unit_y(),
            position: Point3 {
                x: Rad::sin(self.rotation) * self.xz_radius,
                y: self.y_height,
                z: Rad::cos(self.rotation) * self.xz_radius,
            },
            target: Point3::origin(),
            projection: PerspectiveFov {
                aspect: aspect_ratio,
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
    pub radius: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}

impl FirstPersonCamera {
    pub fn update(&mut self, _delta_time: f32) {}

    pub fn reset_motion(&mut self) {}

    pub fn compute(&self, aspect_ratio: f32) -> ComputedCamera {
        let camera = Camera {
            up: self.location.up(),
            position: self.location.to_point(self.radius + self.height),
            // TODO: Should keep track of this!
            target: Point3::origin(),
            projection: PerspectiveFov {
                aspect: aspect_ratio,
                fovy: Rad::full_turn() / 6.0,
                near: self.near,
                far: self.far,
            },
        };

        camera.compute()
    }
}
