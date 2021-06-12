use crate::geometry::{Point3f, Triangle2};
use crate::math::vector::VecElem;
use crate::math::{Matrix3, Matrix3f, Matrix4f, Vec3f, Vec4f};
use std::ops::Mul;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform(Matrix4f);

impl Transform {
    pub fn rotation_x(theta: f32) -> Self {
        Matrix3f::rotation_x(theta).into()
    }

    pub fn rotation_y(theta: f32) -> Self {
        Matrix3f::rotation_y(theta).into()
    }

    pub fn rotation_z(theta: f32) -> Self {
        Matrix3f::rotation_z(theta).into()
    }

    pub fn rotation(theta: f32, a: Vec3f) -> Self {
        Matrix3f::rotation(theta, a).into()
    }
}

impl From<Matrix4f> for Transform {
    fn from(m: Matrix4f) -> Self {
        Self(m)
    }
}

impl From<Matrix3f> for Transform {
    fn from(m: Matrix3f) -> Self {
        Self(Matrix4f::from(m))
    }
}

impl Mul<Vec3f> for Transform {
    type Output = Vec3f;

    fn mul(self, rhs: Vec3f) -> Self::Output {
        (self.0 * Vec4f::from(rhs)).xyz()
    }
}

impl Mul<Point3f> for Transform {
    type Output = Point3f;

    fn mul(self, rhs: Point3f) -> Self::Output {
        let v: Vec3f = rhs.into();
        (self.0 * v.xyz1()).xyz().into()
    }
}
