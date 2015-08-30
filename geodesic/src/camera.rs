use na::{self, Iso3, Mat4, Pnt3, PerspMat3, Vec3};

pub struct Camera<T = f32> {
    pub target: Pnt3<T>,
    pub position: Pnt3<T>,
    pub projection: PerspMat3<T>,
}

impl<T: na::BaseFloat> Camera<T> {
    pub fn view_mat(&self) -> Mat4<T> {
        let mut view: Iso3<T> = na::one();
        view.look_at_z(&self.position, &self.target, &Vec3::z());
        na::to_homogeneous(&na::inv(&view).unwrap())
    }

    pub fn projection_mat(&self) -> Mat4<T> {
        self.projection.to_mat()
    }

    pub fn to_mat(&self) -> Mat4<T> {
        self.projection_mat() * self.view_mat()
    }
}
