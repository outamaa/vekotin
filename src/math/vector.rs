use crate::math::matrix::Matrix3;
pub use num::{Float, Num, Zero};
use std::iter::FromIterator;
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Neg, Sub, SubAssign};

// General note: Use Copy, pass by value, trust the compiler to optimize. :)
// Iterators used heavily to help with copy paste / macrology for dimensions other than 3

pub trait VecElem: Copy + Num {
    fn as_f32(&self) -> f32;
}
impl VecElem for f32 {
    #[inline(always)]
    fn as_f32(&self) -> f32 {
        *self
    }
}
impl VecElem for i32 {
    #[inline(always)]
    fn as_f32(&self) -> f32 {
        *self as f32
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector<T: VecElem, const N: usize> {
    components: [T; N],
}

impl<T: VecElem, const N: usize> Vector<T, N> {
    pub fn abs(&self) -> T {
        self.length_squared()
    }
}

pub type Vec2<T> = Vector<T, 2>;
pub type Vec2f = Vec2<f32>;
pub type Vec2i = Vec2<i32>;

pub type Vec3<T> = Vector<T, 3>;
pub type Vec3f = Vec3<f32>;
pub type Vec3i = Vec3<i32>;

pub type Vec4<T> = Vector<T, 4>;
pub type Vec4f = Vec4<f32>;
pub type Vec4i = Vec4<i32>;

// https://stackoverflow.com/questions/66832882/generics-partial-specialization-in-rust
// Also, please somebody contact me and show me a better way to do this. :)
pub trait SizeAtLeast<const N: usize> {}

impl<T: VecElem, const N: usize> SizeAtLeast<N> for Vector<T, N> {}
impl<T: VecElem> SizeAtLeast<2> for Vector<T, 3> {}
impl<T: VecElem> SizeAtLeast<2> for Vector<T, 4> {}
impl<T: VecElem> SizeAtLeast<3> for Vector<T, 4> {}

impl<T: VecElem, const N: usize> Zero for Vector<T, N> {
    #[inline]
    fn zero() -> Self {
        Self::constant(Zero::zero())
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.components.iter().all(|&x| x.is_zero())
    }
}

impl<T: VecElem, const N: usize> Vector<T, N> {
    #[inline(always)]
    pub fn constant(value: T) -> Self {
        Self {
            components: [value; N],
        }
    }

    #[inline(always)]
    pub fn x(&self) -> T
    where
        Self: SizeAtLeast<2>,
    {
        self.components[0]
    }
    #[inline(always)]
    pub fn y(&self) -> T
    where
        Self: SizeAtLeast<2>,
    {
        self.components[1]
    }

    #[inline(always)]
    pub fn z(&self) -> T
    where
        Self: SizeAtLeast<3>,
    {
        self.components[2]
    }

    #[inline(always)]
    pub fn w(&self) -> T
    where
        Self: SizeAtLeast<4>,
    {
        self.components[3]
    }

    pub fn xyz(&self) -> Vec3<T>
    where
        Self: SizeAtLeast<3>,
    {
        Vec3::new(self.components[0], self.components[1], self.components[2])
    }

    pub fn xyz0(&self) -> Vec4<T>
    where
        Self: SizeAtLeast<3>,
    {
        Vec4::new(
            self.components[0],
            self.components[1],
            self.components[2],
            T::zero(),
        )
    }

    pub fn xyz1(&self) -> Vec4<T>
    where
        Self: SizeAtLeast<3>,
    {
        Vec4::new(
            self.components[0],
            self.components[1],
            self.components[2],
            T::one(),
        )
    }
}

impl<T: VecElem> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { components: [x, y] }
    }
}

impl<T: VecElem> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Vec3<T> {
        Vec3 {
            components: [x, y, z],
        }
    }
}

impl<T: VecElem> Vec4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Vec4<T> {
        Vec4 {
            components: [x, y, z, w],
        }
    }
}

impl<T: VecElem, const N: usize> Vector<T, N> {
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
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
    pub fn iter(&self) -> VectorIterator<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> VectorMutIterator<T> {
        self.into_iter()
    }
}

impl<T: VecElem, const N: usize> Vector<T, N> {
    /// Returns the dot - or inner - product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
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
    pub fn dot(&self, other: Self) -> T {
        let mut sum = T::zero();
        for (c_self, c_other) in self.iter().zip(other.iter()) {
            sum = sum + c_self * c_other;
        }
        sum
    }

    pub fn length_squared(&self) -> T {
        self.dot(*self)
    }

    pub fn length(&self) -> f32 {
        self.length_squared().as_f32().sqrt()
    }

    pub fn as_f32(&self) -> Vector<f32, N> {
        let mut x = Vector::<f32, N>::zero();
        for i in 0..N {
            x[i] = self.components[i].as_f32();
        }
        x
    }

    pub fn unit(&self) -> Vector<f32, N> {
        let length_inv = 1.0 / self.length();
        let mut x = Vector::<f32, N>::zero();
        for i in 0..N {
            x[i] = self.components[i].as_f32() * length_inv;
        }
        x
    }
}

impl<T: VecElem> Vec3<T> {
    /// Returns the outer product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
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
    /// use vekotin::math::vector::*;
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

//
// Arithmetic
//

impl<T: VecElem, const N: usize> Add for Vector<T, N> {
    type Output = Vector<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
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
        let mut x = Self::zero();
        for i in 0..N {
            x[i] = self.components[i] + rhs.components[i];
        }
        x
    }
}

impl<T: VecElem> AddAssign for Vec3<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (i, c) in self.iter_mut().enumerate() {
            *c = *c + rhs[i];
        }
    }
}

impl<T: VecElem, const N: usize> Sub for Vector<T, N> {
    type Output = Vector<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let zero = Vec3f::zero();
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(i - zero, i);
    /// assert_eq!(i - i , zero);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        let mut x = Self::zero();
        for i in 0..N {
            x[i] = self.components[i] - rhs.components[i];
        }
        x
    }
}

impl<T: VecElem, const N: usize> SubAssign for Vector<T, N> {
    fn sub_assign(&mut self, rhs: Self) {
        for (i, c) in self.iter_mut().enumerate() {
            *c = *c - rhs[i];
        }
    }
}

impl<T: VecElem + Neg<Output = T>, const N: usize> Neg for Vector<T, N> {
    type Output = Vector<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let zero = Vec4f::zero();
    /// let i = Vec4f::new(1.0, 0.0, 0.0, 0.0);
    ///
    /// assert_eq!(-zero, zero);
    /// assert_eq!(-i, Vec4f::new(-1.0, 0.0, 0.0, 0.0));
    /// ```
    fn neg(self) -> Self::Output {
        let mut x = Self::zero();
        for i in 0..N {
            x[i] = -self.components[i];
        }
        x
    }
}

impl<T: VecElem, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Vector<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let v = Vec2f::new(1.0, 2.0);
    ///
    /// assert_eq!(v * 2.0, Vec2f::new(2.0, 4.0));
    /// ```
    fn mul(self, rhs: T) -> Self::Output {
        let mut x = Self::zero();
        for i in 0..N {
            x[i] = self.components[i] * rhs;
        }
        x
    }
}

impl<const N: usize> Mul<Vector<f32, N>> for f32 {
    type Output = Vector<f32, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(2.0 * v, Vec3f::new(2.0, 4.0, 6.0));
    /// ```
    fn mul(self, rhs: Vector<f32, N>) -> Self::Output {
        rhs * self
    }
}

impl<const N: usize> Mul<Vector<i32, N>> for i32 {
    type Output = Vector<i32, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let v = Vec4i::new(1, 2, 3, 4);
    ///
    /// assert_eq!(2 * v, Vec4i::new(2, 4, 6, 8));
    /// ```
    fn mul(self, rhs: Vector<i32, N>) -> Self::Output {
        rhs * self
    }
}

impl<T: VecElem + Div<Output = T>, const N: usize> Div<T> for Vector<T, N> {
    type Output = Vector<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let v = Vec4f::new(1.0, 2.0, 3.0, 4.0);
    ///
    /// assert_eq!(v / 2.0, Vec4f::new(0.5, 1.0, 1.5, 2.0));
    /// ```
    fn div(self, rhs: T) -> Self::Output {
        let inv_rhs = T::one() / rhs;
        let mut x = Self::zero();
        for i in 0..N {
            x[i] = self.components[i] * inv_rhs;
        }
        x
    }
}

//
// Iterators
//

pub struct VectorIterator<'a, T> {
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T: VecElem, const N: usize> IntoIterator for &'a Vector<T, N> {
    type Item = T;
    type IntoIter = VectorIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VectorIterator {
            iter: self.components.iter(),
        }
    }
}

impl<'a, T: VecElem> Iterator for VectorIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(next) => Some(*next),
            _ => None,
        }
    }
}

pub struct VectorMutIterator<'a, T> {
    iter: std::slice::IterMut<'a, T>,
}

impl<'a, T: VecElem, const N: usize> IntoIterator for &'a mut Vector<T, N> {
    type Item = &'a mut T;
    type IntoIter = VectorMutIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VectorMutIterator {
            iter: self.components.iter_mut(),
        }
    }
}

impl<'a, T> Iterator for VectorMutIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: VecElem, const N: usize> FromIterator<T> for Vector<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v: Vector<T, N> = Vector::zero();
        for (i, c) in iter.into_iter().take(N).enumerate() {
            v.components[i] = c;
        }
        v
    }
}

//
// Indexing
//

impl<T: VecElem, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    /// # Examples
    /// ```rust
    /// use vekotin::math::vector::*;
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

impl<T: VecElem, const N: usize> IndexMut<usize> for Vector<T, N> {
    /// # Examples
    /// ```rust
    /// use vekotin::math::vector::*;
    ///
    /// let mut v = Vec3f::new(1.0, 2.0, 3.0);
    /// v[0] = 2.0;
    /// assert_eq!(v[0], 2.0);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}

impl<T: VecElem> From<Vec3<T>> for Vec4<T> {
    fn from(v: Vec3<T>) -> Self {
        Self::new(v.x(), v.y(), v.z(), T::zero())
    }
}
