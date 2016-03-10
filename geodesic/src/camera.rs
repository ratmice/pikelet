use cgmath::{BaseFloat, Matrix4, Point3, Vector3, PerspectiveFov};

pub struct Camera<T = f32> {
    pub target: Point3<T>,
    pub position: Point3<T>,
    pub projection: PerspectiveFov<T>,
}

impl<T: BaseFloat> Camera<T> {
    pub fn view_matrix(&self) -> Matrix4<T> {
        Matrix4::look_at(self.position, self.target, Vector3::unit_y())
    }

    pub fn projection_matrix(&self) -> Matrix4<T> {
        self.projection.into()
    }
}
