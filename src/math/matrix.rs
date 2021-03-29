use crate::math::vec::{Vec3, VecElem, Vector};
use num::{Float, Zero};
use std::iter::FromIterator;
use std::ops::{Add, Mul, Sub};
use std::slice::Iter;

// Note: COLUMN major data layout, but usual row major indexing with get

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix<T: VecElem, const N: usize> {
    columns: [Vector<T, N>; N],
}

pub type Matrix2<T> = Matrix<T, 2>;
pub type Matrix2f = Matrix2<f32>;
pub type Matrix2i = Matrix2<i32>;

pub type Matrix3<T> = Matrix<T, 3>;
pub type Matrix3f = Matrix3<f32>;
pub type Matrix3i = Matrix3<i32>;

pub type Matrix4<T> = Matrix<T, 4>;
pub type Matrix4f = Matrix4<f32>;
pub type Matrix4i = Matrix4<i32>;

impl<T: VecElem> Matrix3<T> {
    //
    // Constructors
    //

    pub fn new(
        m00: T,
        m01: T,
        m02: T,
        m10: T,
        m11: T,
        m12: T,
        m20: T,
        m21: T,
        m22: T,
    ) -> Matrix3<T> {
        Matrix3 {
            columns: [
                Vec3::new(m00, m10, m20),
                Vec3::new(m01, m11, m21),
                Vec3::new(m02, m12, m22),
            ],
        }
    }

    pub fn from_columns(x: Vec3<T>, y: Vec3<T>, z: Vec3<T>) -> Matrix3<T> {
        Matrix3 { columns: [x, y, z] }
    }

    pub fn from_rows(x: Vec3<T>, y: Vec3<T>, z: Vec3<T>) -> Matrix3<T> {
        Matrix3::new(x[0], x[1], x[2], y[0], y[1], y[2], z[0], z[1], z[2])
    }

    //
    // Getters, setters, iterators
    //

    pub fn get(&self, row: usize, col: usize) -> T {
        self.columns[col][row]
    }
    pub fn set(&mut self, row: usize, col: usize, val: T) -> &mut Self {
        self.columns[col][row] = val;
        self
    }

    pub fn row(&self, row: usize) -> Vec3<T> {
        Vec3::new(self.get(row, 0), self.get(row, 1), self.get(row, 2))
    }

    pub fn col(&self, col: usize) -> Vec3<T> {
        self.columns[col]
    }

    pub fn columns(&self) -> Matrix3Iterator<T> {
        self.into_iter()
    }

    pub fn columns_mut(&mut self) -> Matrix3MutIterator<T> {
        self.into_iter()
    }

    //
    // Basic matrix operations
    //

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::identity();
    ///
    /// assert_eq!(zero.transpose(), zero);
    /// assert_eq!(id.transpose(), id);
    /// assert_eq!(Matrix3f::new(1.0, 2.0, 3.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0).transpose(),
    ///            Matrix3f::new(1.0, 0.0, 0.0, 2.0, 2.0, 0.0, 3.0, 0.0, 2.0));
    /// ```
    pub fn transpose(&self) -> Matrix3<T> {
        Matrix3::new(
            self.get(0, 0),
            self.get(1, 0),
            self.get(2, 0),
            self.get(0, 1),
            self.get(1, 1),
            self.get(2, 1),
            self.get(0, 2),
            self.get(1, 2),
            self.get(2, 2),
        )
    }
}

impl<T: Float + VecElem + Mul<Matrix3<T>, Output = Matrix3<T>>> Matrix3<T> {
    /// Given vector `a`, return a matrix that, when multiplied with vector `v` returns the same
    /// result as `a.cross(v)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    /// use vekotin::math::vec::Vec3f;
    ///
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(i.cross(j), Matrix3f::cross(i) * j);
    /// ```
    pub fn cross(a: Vec3<T>) -> Matrix3<T> {
        Matrix3::new(
            T::zero(),
            -a.z(),
            a.y(),
            a.z(),
            T::zero(),
            -a.z(),
            -a.y(),
            a.x(),
            T::zero(),
        )
    }

    pub fn rotation_x(theta: T) -> Matrix3<T> {
        Matrix3::rotation(theta, Vec3::new(T::one(), T::zero(), T::zero()))
    }

    pub fn rotation_y(theta: T) -> Matrix3<T> {
        Matrix3::rotation(theta, Vec3::new(T::zero(), T::one(), T::zero()))
    }

    pub fn rotation_z(theta: T) -> Matrix3<T> {
        Matrix3::rotation(theta, Vec3::new(T::zero(), T::zero(), T::one()))
    }

    pub fn rotation(theta: T, a: Vec3<T>) -> Matrix3<T> {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        // To rotate v about a by theta we want
        // v' = cos_theta * v + (1 - cos_theta) * (v . a) * a + sin_theta * (a x v)
        // Here we just squash what's done to v on the right hand side into a single matrix
        cos_theta * Matrix3::identity()
            + (T::one() - cos_theta) * a.outer(a)
            + sin_theta * Matrix3::cross(a)
    }

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::{Matrix3, Matrix3f};
    ///
    /// let zero: Matrix3f = Matrix3::zero();
    /// let id: Matrix3f = Matrix3::identity();
    /// let ortho = Matrix3::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    ///
    /// assert_eq!(zero.inverse(), None);
    /// assert_eq!(id.inverse().unwrap(), id);
    /// assert_eq!(ortho.inverse().unwrap(), ortho.transpose());
    /// ```
    pub fn inverse(&self) -> Option<Matrix3<T>> {
        let a = self.col(0);
        let b = self.col(1);
        let c = self.col(2);

        let r0 = b.cross(c);
        let r1 = c.cross(a);
        let r2 = a.cross(b);

        let det = r2.dot(c);
        if det == T::zero() {
            None
        } else {
            let inv_det = T::one() / det;
            Some(Matrix3::from_rows(r0 * inv_det, r1 * inv_det, r2 * inv_det))
        }
    }
}

impl<T: VecElem> Matrix3<T> {
    pub fn zero() -> Matrix3<T> {
        Matrix3::new(
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
        )
    }
}

impl<T: VecElem> Matrix3<T> {
    pub fn identity() -> Matrix3<T> {
        Matrix3::new(
            T::one(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
        )
    }
}

impl<T: VecElem + Sub<Output = T>> Matrix3<T> {
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::identity();
    ///
    /// assert_eq!(zero.determinant(), 0.0);
    /// assert_eq!(id.determinant(), 1.0);
    /// assert_eq!(Matrix3f::new(1.0, 2.0, 3.0, 2.0, 3.0, 4.0, -3.0, 4.0, 5.0).determinant(), 6.0);
    /// ```
    pub fn determinant(&self) -> T {
        self.get(0, 0) * (self.get(1, 1) * self.get(2, 2) - self.get(2, 1) * self.get(1, 2))
            + self.get(0, 1) * (self.get(1, 2) * self.get(2, 0) - self.get(1, 0) * self.get(2, 2))
            + self.get(0, 2) * (self.get(1, 0) * self.get(2, 1) - self.get(1, 1) * self.get(2, 0))
    }
}

impl<T: VecElem> Matrix3<T> {
    /// Checks if the matrix is orthogonal by checking if M^T * M == I
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::identity();
    /// let ortho = Matrix3f::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    /// let unortho = Matrix3f::new(0.0, 2.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    ///
    /// assert!(!zero.is_orthogonal());
    /// assert!(id.is_orthogonal());
    /// assert!(ortho.is_orthogonal());
    /// assert!(!unortho.is_orthogonal());
    /// ```
    pub fn is_orthogonal(&self) -> bool {
        (*self) * self.transpose() == Matrix3::identity()
    }
}

//
// Arithmetic
//

impl<T: VecElem> Add for Matrix3<T> {
    type Output = Matrix3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::identity();
    ///
    /// assert_eq!(zero + id, id);
    /// assert_eq!(id + zero, id);
    /// assert_eq!(zero + zero, zero);
    /// assert_eq!(id + id, Matrix3f::new(2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0));
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        self.columns()
            .zip(rhs.columns())
            .map(|(a, b)| *a + *b)
            .collect()
    }
}

impl<T: VecElem> Mul for Matrix3<T> {
    type Output = Matrix3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::identity();
    ///
    /// assert_eq!(zero * id, zero);
    /// assert_eq!(id * zero, zero);
    /// assert_eq!(zero * zero, zero);
    /// assert_eq!(id * id, id);
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        let mut m = Matrix3::zero();
        for i in 0 as usize..3 {
            for j in 0 as usize..3 {
                m.set(i, j, self.row(i).dot(rhs.col(j)));
            }
        }
        m
    }
}

impl<T: VecElem> Mul<Vec3<T>> for Matrix3<T> {
    type Output = Vec3<T>;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3f;
    /// use vekotin::math::vec::Vec3f;
    /// use num::Zero;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::identity();
    /// let flip = Matrix3f::new(0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0);
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    /// let v_zero = Vec3f::zero();
    ///
    /// assert_eq!(zero * v, v_zero);
    /// assert_eq!(id * v, v);
    /// assert_eq!(flip * v, Vec3f::new(3.0, 2.0, 1.0));
    /// ```
    fn mul(self, rhs: Vec3<T>) -> Self::Output {
        self.columns()
            .zip(rhs.iter())
            .map(|(col, c)| (*col) * c)
            .fold(Vec3::zero(), |acc, v| acc + v)
    }
}

impl<T: VecElem> Mul<T> for &Matrix3<T> {
    type Output = Matrix3<T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.columns().map(|col| (*col) * rhs).collect()
    }
}

impl<T: VecElem> Mul<T> for Matrix3<T> {
    type Output = Matrix3<T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.columns().map(|col| (*col) * rhs).collect()
    }
}

impl Mul<&Matrix3<f32>> for f32 {
    type Output = Matrix3<f32>;

    fn mul(self, rhs: &Matrix3<f32>) -> Self::Output {
        rhs * self
    }
}

impl Mul<Matrix3<f32>> for f32 {
    type Output = Matrix3<f32>;

    fn mul(self, rhs: Matrix3<f32>) -> Self::Output {
        rhs * self
    }
}

//
// Iterators
//

pub struct Matrix3Iterator<'a, T: VecElem> {
    iter: Iter<'a, Vec3<T>>,
}

impl<'a, T: VecElem> IntoIterator for &'a Matrix3<T> {
    type Item = &'a Vec3<T>;
    type IntoIter = Matrix3Iterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Matrix3Iterator {
            iter: self.columns.iter(),
        }
    }
}

impl<'a, T: VecElem> Iterator for Matrix3Iterator<'a, T> {
    type Item = &'a Vec3<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct Matrix3MutIterator<'a, T: VecElem> {
    iter: std::slice::IterMut<'a, Vec3<T>>,
}

impl<'a, T: VecElem> IntoIterator for &'a mut Matrix3<T> {
    type Item = &'a mut Vec3<T>;
    type IntoIter = Matrix3MutIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Matrix3MutIterator {
            iter: self.columns.iter_mut(),
        }
    }
}

impl<'a, T: VecElem> Iterator for Matrix3MutIterator<'a, T> {
    type Item = &'a mut Vec3<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: VecElem> FromIterator<Vec3<T>> for Matrix3<T> {
    fn from_iter<I: IntoIterator<Item = Vec3<T>>>(iter: I) -> Self {
        let mut v_iter = iter.into_iter().take(3);
        let x = v_iter.next().unwrap_or(Vec3::zero());
        let y = v_iter.next().unwrap_or(Vec3::zero());
        let z = v_iter.next().unwrap_or(Vec3::zero());
        Matrix3 { columns: [x, y, z] }
    }
}
