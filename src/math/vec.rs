use crate::math::matrix::Matrix3;
use num::{Float, Num};
use std::iter::FromIterator;
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Neg, Sub, SubAssign};

// General note: Use Copy, pass by value, trust the compiler to optimize. :)
// Iterators used heavily to help with copy paste / macrology for dimensions other than 3

pub trait VecElem: Copy + Num {}
impl VecElem for f32 {}
impl VecElem for i32 {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3<T: VecElem> {
    components: [T; 3],
}

pub type Vec3f = Vec3<f32>;
pub type Vec3i = Vec3<i32>;

impl<T: VecElem> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Vec3<T> {
        Vec3 {
            components: [x, y, z],
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    /// let mut v_iter = v.iter();
    /// assert_eq!(1.0, v_iter.next().unwrap());
    /// assert_eq!(2.0, v_iter.next().unwrap());
    /// assert_eq!(3.0, v_iter.next().unwrap());
    /// assert!(v_iter.next().is_none());
    ///
    /// let v2 = v.iter().collect();
    /// assert_eq!(v, v2);
    /// ```
    pub fn iter(&self) -> Vec3DIterator<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> Vec3DMutIterator<T> {
        self.into_iter()
    }

    pub fn x(&self) -> T {
        self.components[0]
    }
    pub fn y(&self) -> T {
        self.components[1]
    }
    pub fn z(&self) -> T {
        self.components[2]
    }
}

impl<T: VecElem> Vec3<T> {
    /// Returns the dot - or inner - product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(1.0, i.dot(i));
    /// assert_eq!(0.0, j.dot(i));
    /// assert_eq!(0.0, i.dot(j));
    /// assert_eq!(1.0, k.dot(k));
    /// ```
    pub fn dot(&self, other: Vec3<T>) -> T {
        let mut sum = T::zero();
        for (c_self, c_other) in self.iter().zip(other.iter()) {
            sum = sum + c_self * c_other;
        }
        sum
    }

    pub fn length_squared(&self) -> T {
        self.dot(*self)
    }

    /// Returns the outer product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    /// use vekotin::math::matrix::Matrix3f;
    ///
    /// let zero = Vec3f::zero();
    /// let m_zero = Matrix3f::zero();
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(m_zero, zero.outer(zero));
    /// assert_eq!(m_zero, zero.outer(i));
    /// assert_eq!(j.outer(k),
    ///            Matrix3f::new(0.0, 0.0, 0.0,
    ///                          0.0, 0.0, 1.0,
    ///                          0.0, 0.0, 0.0));
    /// ```
    pub fn outer(&self, other: Vec3<T>) -> Matrix3<T> {
        let mut m = Matrix3::zero();
        for (i, a) in self.iter().enumerate() {
            for (j, b) in other.iter().enumerate() {
                m.set(i, j, a * b);
            }
        }
        m
    }

    /// Returns the cross product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3f::zero();
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(zero, i.cross(i));
    /// assert_eq!(i, j.cross(k));
    /// assert_eq!(j, k.cross(i));
    /// assert_eq!(k, i.cross(j));
    /// ```
    pub fn cross(&self, other: Vec3<T>) -> Vec3<T> {
        Vec3::new(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }

    pub fn scalar_triple_product(a: Vec3<T>, b: Vec3<T>, c: Vec3<T>) -> T {
        a.cross(b).dot(c)
    }

    pub fn vector_triple_product(a: Vec3<T>, b: Vec3<T>, c: Vec3<T>) -> Vec3<T> {
        a.cross(b.cross(c))
    }
}

impl<T: VecElem> Vec3<T> {
    /// Returns a 3D vector with all components set as zeroes
    pub fn zero() -> Vec3<T> {
        Vec3::new(T::zero(), T::zero(), T::zero())
    }
}

impl<T: VecElem + Float> Vec3<T> {
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn unit(&self) -> Vec3<T> {
        *self / self.length()
    }
}

//
// Arithmetic
//

impl<T: VecElem> Add for Vec3<T> {
    type Output = Vec3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3f::zero();
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(zero + i, i);
    /// assert_eq!(i + zero, i);
    /// assert_eq!(i + j, j + i);
    /// assert_eq!(i + j + k, k + j + i);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        self.iter().zip(rhs.iter()).map(|(a, b)| a + b).collect()
    }
}

impl<T: VecElem> AddAssign for Vec3<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (i, c) in self.iter_mut().enumerate() {
            *c = *c + rhs[i];
        }
    }
}

impl<T: VecElem> Sub for Vec3<T> {
    type Output = Vec3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3f::zero();
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(i - zero, i);
    /// assert_eq!(i - i , zero);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        self.iter().zip(rhs.iter()).map(|(a, b)| a - b).collect()
    }
}

impl<T: VecElem> SubAssign for Vec3<T> {
    fn sub_assign(&mut self, rhs: Self) {
        for (i, c) in self.iter_mut().enumerate() {
            *c = *c - rhs[i];
        }
    }
}

impl<T: VecElem + Neg<Output = T>> Neg for Vec3<T> {
    type Output = Vec3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3f::zero();
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(-zero, zero);
    /// assert_eq!(-i, Vec3f::new(-1.0, 0.0, 0.0));
    /// ```
    fn neg(self) -> Self::Output {
        self.iter().map(|a| -a).collect()
    }
}

impl<T: VecElem> Mul<T> for Vec3<T> {
    type Output = Vec3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(v * 2.0, Vec3f::new(2.0, 4.0, 6.0));
    /// ```
    fn mul(self, rhs: T) -> Self::Output {
        self.iter().map(|a| rhs * a).collect()
    }
}

impl Mul<Vec3<f32>> for f32 {
    type Output = Vec3<f32>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(2.0 * v, Vec3f::new(2.0, 4.0, 6.0));
    /// ```
    fn mul(self, rhs: Vec3<f32>) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vec3<i32>> for i32 {
    type Output = Vec3<i32>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3i::new(1, 2, 3);
    ///
    /// assert_eq!(2 * v, Vec3i::new(2, 4, 6));
    /// ```
    fn mul(self, rhs: Vec3<i32>) -> Self::Output {
        rhs * self
    }
}

impl<T: VecElem + Div<Output = T>> Div<T> for Vec3<T> {
    type Output = Vec3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(v / 2.0, Vec3f::new(0.5, 1.0, 1.5));
    /// ```
    fn div(self, rhs: T) -> Self::Output {
        let inv_rhs = T::one() / rhs;
        self.iter().map(|a| a * inv_rhs).collect()
    }
}

impl Div<Vec3<f32>> for f32 {
    type Output = Vec3<f32>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(6.0 / v, Vec3f::new(6.0, 3.0, 2.0));
    /// ```
    fn div(self, rhs: Vec3<f32>) -> Self::Output {
        rhs.iter().map(|a| self / a).collect()
    }
}

//
// Iterators
//

pub struct Vec3DIterator<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T: VecElem> IntoIterator for &'a Vec3<T> {
    type Item = T;
    type IntoIter = Vec3DIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Vec3DIterator {
            iter: self.components.iter(),
        }
    }
}

impl<'a, T: VecElem> Iterator for Vec3DIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(next) => Some(*next),
            _ => None,
        }
    }
}

pub struct Vec3DMutIterator<'a, T> {
    iter: std::slice::IterMut<'a, T>,
}

impl<'a, T: VecElem> IntoIterator for &'a mut Vec3<T> {
    type Item = &'a mut T;
    type IntoIter = Vec3DMutIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Vec3DMutIterator {
            iter: self.components.iter_mut(),
        }
    }
}

impl<'a, T> Iterator for Vec3DMutIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: VecElem> FromIterator<T> for Vec3<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v_iter = iter.into_iter().take(3);
        let x = v_iter.next().unwrap_or(T::zero());
        let y = v_iter.next().unwrap_or(T::zero());
        let z = v_iter.next().unwrap_or(T::zero());
        Vec3 {
            components: [x, y, z],
        }
    }
}

//
// Indexing
//

impl<T: VecElem> Index<usize> for Vec3<T> {
    type Output = T;

    /// # Examples
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    /// assert_eq!(v[0], 1.0);
    /// assert_eq!(v[1], 2.0);
    /// assert_eq!(v[2], 3.0);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.components[index]
    }
}

impl<T: VecElem> IndexMut<usize> for Vec3<T> {
    /// # Examples
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let mut v = Vec3f::new(1.0, 2.0, 3.0);
    /// v[0] = 2.0;
    /// assert_eq!(v[0], 2.0);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}
