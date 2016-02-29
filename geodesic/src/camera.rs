use cgmath::{BaseFloat, Matrix4, Point3, Vector3, PerspectiveFov};

pub struct Camera<T = f32> {
    pub target: Point3<T>,
    pub position: Point3<T>,
    pub projection: PerspectiveFov<T>,
}

impl<T: BaseFloat> Camera<T> {
    pub fn view_mat(&self) -> Matrix4<T> {
        Matrix4::look_at(self.position, self.target, Vector3::unit_z())
    }

    pub fn projection_mat(&self) -> Matrix4<T> {
        self.projection.into()
    }

    pub fn to_mat(&self) -> Matrix4<T> {
        &self.projection_mat() * &self.view_mat()
    }
}
