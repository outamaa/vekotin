use crate::math::vec::Vec3D;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix3D {
    columns: [Vec3D; 3],
}
