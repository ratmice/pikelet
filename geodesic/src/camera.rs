use na::{self, Iso3, Mat4, Pnt3, PerspMat3, Vec3};
use std::f32;

pub struct Camera {
    pub target: Pnt3<f32>,
    pub position: Pnt3<f32>,
    pub near: f32,
    pub far: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
}

pub const DEFAULT: Camera = Camera {
    target: Pnt3 { x: 0.0, y: 0.0, z: 0.0 },
    position: Pnt3 { x: 0.0, y: 0.0, z: 0.0 },
    near: 0.1,
    far: 300.0,
    fov: f32::consts::PI / 4.0,
    aspect_ratio: 1.0,
};

impl Camera {
    pub fn view_mat(&self) -> Mat4<f32> {
        let mut view: Iso3<f32> = na::one();
        view.look_at_z(&self.position, &self.target, &Vec3::z());
        na::to_homogeneous(&na::inv(&view).unwrap())
    }

    pub fn projection_mat(&self) -> Mat4<f32> {
        PerspMat3::new(self.aspect_ratio, self.fov, self.near, self.far).to_mat()
    }

    pub fn to_mat(&self) -> Mat4<f32> {
        self.projection_mat() * self.view_mat()
    }
}
