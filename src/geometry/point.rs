use crate::math::vector::{VecElem, Vector};
use crate::math::{Vec2, Vec3};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point<T: VecElem, const N: usize>(Vector<T, N>);

pub type Point2<T> = Point<T, 2>;
pub type Point2f = Point2<f32>;
pub type Point2i = Point2<i32>;

pub type Point3<T> = Point<T, 3>;
pub type Point3f = Point3<f32>;
pub type Point3i = Point3<i32>;

pub type Point4<T> = Point<T, 4>;
pub type Point4f = Point4<f32>;
pub type Point4i = Point4<i32>;

impl<T: VecElem> Point2<T> {
    pub fn new(x: T, y: T) -> Self {
        Point(Vec2::<T>::new(x, y))
    }
    #[inline]
    pub fn x(&self) -> T {
        self.0.x()
    }
    #[inline]
    pub fn y(&self) -> T {
        self.0.y()
    }
}

impl<T: VecElem> Point3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Point(Vec3::<T>::new(x, y, z))
    }
    #[inline]
    pub fn x(&self) -> T {
        self.0.x()
    }
    #[inline]
    pub fn y(&self) -> T {
        self.0.y()
    }
    #[inline]
    pub fn z(&self) -> T {
        self.0.z()
    }
}

impl<T: VecElem, const N: usize> Add<Vector<T, N>> for Point<T, N> {
    type Output = Point<T, N>;

    fn add(self, rhs: Vector<T, N>) -> Self::Output {
        Point(self.0 + rhs)
    }
}

impl<T: VecElem, const N: usize> Add<Point<T, N>> for Vector<T, N> {
    type Output = Point<T, N>;

    fn add(self, rhs: Point<T, N>) -> Self::Output {
        Point(self + rhs.0)
    }
}

impl<T: VecElem, const N: usize> Sub for Point<T, N> {
    type Output = Vector<T, N>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl<T: VecElem, const N: usize> From<Vector<T, N>> for Point<T, N> {
    fn from(v: Vector<T, N>) -> Self {
        Self(v)
    }
}

impl<T: VecElem, const N: usize> Into<Vector<T, N>> for Point<T, N> {
    fn into(self) -> Vector<T, N> {
        self.0
    }
}
