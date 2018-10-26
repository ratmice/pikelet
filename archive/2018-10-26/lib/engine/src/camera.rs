use cgmath::{Matrix4, PerspectiveFov, Point3, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub up: Vector3<f32>,
    pub target: Point3<f32>,
    pub position: Point3<f32>,
    pub projection: PerspectiveFov<f32>,
}

impl Camera {
    pub fn compute_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.position, self.target, self.up)
    }

    pub fn compute_projection_matrix(&self) -> Matrix4<f32> {
        self.projection.into()
    }

    pub fn compute(&self) -> ComputedCamera {
        ComputedCamera {
            view: self.compute_view_matrix(),
            position: self.position,
            projection: self.compute_projection_matrix(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComputedCamera {
    pub view: Matrix4<f32>,
    pub position: Point3<f32>,
    pub projection: Matrix4<f32>,
}
