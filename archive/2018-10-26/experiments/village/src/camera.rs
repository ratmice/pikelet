// Copyright The Voyager Developers 2014

// Based on https://github.com/PistonDevelopers/cam/blob/master/src/camera.rs

// use std::num::Float;
use nalgebra::*;

#[derive(Copy)]
pub struct Camera<N = f32> {
    pub view: Iso3<N>,
    pub proj: PerspMat3<N>,
}

impl<N: BaseFloat> Camera<N> {
    pub fn new(translation: Vec3<N>, proj: PerspMat3<N>) -> Camera<N> {
        Camera {
            view: Iso3::new(translation, one()),
            proj: proj,
        }
    }

    pub fn look_at(&mut self, eye: &Pnt3<N>, at: &Pnt3<N>, up: &Vec3<N>) {
        self.view.look_at_z(eye, at, up)
    }

    /// Calculate the matrix transformation of this camera. It is advisable to
    /// store this if using it multiple times so that redundant calculations are
    /// not performed.
    pub fn to_mat(&self) -> Mat4<N> {
        self.proj.to_mat() * to_homogeneous(&inv(&self.view).unwrap())
    }
}
