use na::Mat4;

pub fn scale_mat4(scale: f32) -> Mat4<f32> {
    Mat4::new(
        scale, 0.0, 0.0, 0.0,
        0.0, scale, 0.0, 0.0,
        0.0, 0.0, scale, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}
