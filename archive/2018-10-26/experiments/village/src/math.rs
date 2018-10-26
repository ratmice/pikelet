// Copyright The Voyager Developers 2014

use nalgebra::*;

/// Construct a model matrix
pub fn model_mat(scale: Vec3<f32>, position: Pnt3<f32>) -> Mat4<f32> {
    let mut model: Mat4<f32> = zero();
    model.set_col(0, Vec4::x() * scale.x);
    model.set_col(1, Vec4::y() * scale.y);
    model.set_col(2, Vec4::z() * scale.z);
    model.set_col(3, position.to_homogeneous().to_vec());
    model
}
