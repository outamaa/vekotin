use crate::math::vec::{Vec3, VecElem};
use std::ops::{Add, Sub};

pub struct Point3<T: VecElem>(Vec3<T>);

pub type Point3f = Point3<f32>;
pub type Point3i = Point3<i32>;

impl<T: VecElem> Add<Vec3<T>> for Point3<T> {
    type Output = Point3<T>;

    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Point3(self.0 + rhs)
    }
}

impl<T: VecElem> Add<Point3<T>> for Vec3<T> {
    type Output = Point3<T>;

    fn add(self, rhs: Point3<T>) -> Self::Output {
        Point3(self + rhs.0)
    }
}

impl<T: VecElem> Sub for Point3<T> {
    type Output = Vec3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl<T: VecElem> From<Vec3<T>> for Point3<T> {
    fn from(v: Vec3<T>) -> Self {
        Self(v)
    }
}

impl<T: VecElem> Into<Vec3<T>> for Point3<T> {
    fn into(self) -> Vec3<T> {
        self.0
    }
}
