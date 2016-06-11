use cgmath::{Matrix4, Point3, Vector3, PerspectiveFov};

pub struct Camera {
    pub target: Point3<f32>,
    pub position: Point3<f32>,
    pub projection: PerspectiveFov<f32>,
}

impl Camera {
    pub fn compute_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.position, self.target, Vector3::unit_y())
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

pub struct ComputedCamera {
    pub view: Matrix4<f32>,
    pub position: Point3<f32>,
    pub projection: Matrix4<f32>,
}
