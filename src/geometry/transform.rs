use crate::geometry::{Point3f, Point4f, Triangle2};
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

    pub fn translation(a: Vec3f) -> Self {
        Matrix4f::new(
            1.0,
            0.0,
            0.0,
            a.x(),
            0.0,
            1.0,
            0.0,
            a.y(),
            0.0,
            0.0,
            1.0,
            a.z(),
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .into()
    }

    pub fn frustum_projection(fov_y: f32, s: f32, n: f32, f: f32) -> Self {
        let g = 1.0 / (fov_y * 0.5).tan();
        let k = f / (f - n);

        Matrix4f::new(
            g / s,
            0.0,
            0.0,
            0.0,
            0.0,
            g,
            0.0,
            0.0,
            0.,
            0.0,
            k,
            -n * k,
            0.0,
            0.0,
            1.0,
            0.0,
        )
        .into()
    }

    pub fn infinite_projection(fov_y: f32, s: f32, n: f32, e: f32) -> Self {
        let g = 1.0 / (fov_y * 0.5).tan();
        let e = 1.0 - e;

        Matrix4f::new(
            g / s,
            0.0,
            0.0,
            0.0,
            0.0,
            g,
            0.0,
            0.0,
            0.0,
            0.0,
            e,
            -n * e,
            0.0,
            0.0,
            1.0,
            0.0,
        )
        .into()
    }

    pub fn rev_frustum_projection(fov_y: f32, s: f32, n: f32, f: f32) -> Self {
        let g = 1.0 / (fov_y * 0.5).tan();
        let k = n / (n - f);

        Matrix4f::new(
            g / s,
            0.0,
            0.0,
            0.0,
            0.0,
            g,
            0.0,
            0.0,
            0.0,
            0.0,
            k,
            -f * k,
            0.0,
            0.0,
            1.0,
            0.0,
        )
        .into()
    }

    pub fn rev_infinite_projection(fov_y: f32, s: f32, n: f32, e: f32) -> Self {
        let g = 1.0 / (fov_y * 0.5).tan();

        Matrix4f::new(
            g / s,
            0.0,
            0.0,
            0.0,
            0.0,
            g,
            0.0,
            0.0,
            0.0,
            0.0,
            e,
            n * (1.0 - e),
            0.0,
            0.0,
            1.0,
            0.0,
        )
        .into()
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
    // Because of perspective divide
    type Output = Point4f;

    fn mul(self, rhs: Point3f) -> Self::Output {
        let v: Vec3f = rhs.into();
        (self.0 * v.xyz1()).into()
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        (self.0 * rhs.0).into()
    }
}
