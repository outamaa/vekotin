use crate::vector::{Vec3, Vec3f, VecElem, Vector};
use crate::{Vec2, Vec4};
pub use num::{Float, One, Zero};
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

impl<T: VecElem, const N: usize> Matrix<T, N> {
    pub fn get(&self, row: usize, col: usize) -> T {
        self.columns[col][row]
    }
    pub fn set(&mut self, row: usize, col: usize, val: T) -> &mut Self {
        self.columns[col][row] = val;
        self
    }

    pub fn row(&self, row: usize) -> Vector<T, N> {
        let mut v: Vector<T, N> = Vector::zero();
        for i in 0..N {
            v[i] = self.get(row, i);
        }
        v
    }

    pub fn col(&self, col: usize) -> Vector<T, N> {
        self.columns[col]
    }

    pub fn columns(&self) -> MatrixIterator<T, N> {
        self.into_iter()
    }

    pub fn columns_mut(&mut self) -> MatrixMutIterator<T, N> {
        self.into_iter()
    }

    //
    // Basic matrix operations
    //

    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::one();
    ///
    /// assert_eq!(zero.transpose(), zero);
    /// assert_eq!(id.transpose(), id);
    /// assert_eq!(Matrix3f::new(1.0, 2.0, 3.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0).transpose(),
    ///            Matrix3f::new(1.0, 0.0, 0.0, 2.0, 2.0, 0.0, 3.0, 0.0, 2.0));
    /// ```
    pub fn transpose(&self) -> Self {
        let mut m = Matrix::zero();
        for row in 0..N {
            for col in 0..N {
                m.set(row, col, self.get(col, row));
            }
        }
        m
    }
}

impl<T: VecElem> Matrix2<T> {
    //
    // Constructors
    //

    pub fn new(m00: T, m01: T, m10: T, m11: T) -> Self {
        Self {
            columns: [Vec2::new(m00, m10), Vec2::new(m01, m11)],
        }
    }

    pub fn from_columns(x: Vec2<T>, y: Vec2<T>) -> Self {
        Self { columns: [x, y] }
    }

    pub fn from_rows(x: Vec2<T>, y: Vec2<T>) -> Self {
        Self::new(x[0], x[1], y[0], y[1])
    }
}

impl<T: VecElem> Matrix3<T> {
    //
    // Constructors
    //
    #[allow(clippy::too_many_arguments)]
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
}

impl<T: VecElem> Matrix4<T> {
    //
    // Constructors
    //
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m00: T,
        m01: T,
        m02: T,
        m03: T,
        m10: T,
        m11: T,
        m12: T,
        m13: T,
        m20: T,
        m21: T,
        m22: T,
        m23: T,
        m30: T,
        m31: T,
        m32: T,
        m33: T,
    ) -> Matrix4<T> {
        Matrix4 {
            columns: [
                Vec4::new(m00, m10, m20, m30),
                Vec4::new(m01, m11, m21, m31),
                Vec4::new(m02, m12, m22, m32),
                Vec4::new(m03, m13, m23, m33),
            ],
        }
    }

    pub fn from_columns(x: Vec4<T>, y: Vec4<T>, z: Vec4<T>, w: Vec4<T>) -> Matrix4<T> {
        Matrix4 {
            columns: [x, y, z, w],
        }
    }

    pub fn from_rows(x: Vec4<T>, y: Vec4<T>, z: Vec4<T>, w: Vec4<T>) -> Matrix4<T> {
        Matrix4::new(
            x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3], z[0], z[1], z[2], z[3], w[0], w[1],
            w[2], w[3],
        )
    }
}

impl<T: VecElem> From<Matrix3<T>> for Matrix4<T> {
    fn from(m: Matrix3<T>) -> Self {
        Matrix4::new(
            m.get(0, 0),
            m.get(0, 1),
            m.get(0, 2),
            T::zero(),
            m.get(1, 0),
            m.get(1, 1),
            m.get(1, 2),
            T::zero(),
            m.get(2, 0),
            m.get(2, 1),
            m.get(2, 2),
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
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
    /// use math::matrix::*;
    /// use math::vector::*;
    ///
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(i.cross(j), Matrix3f::cross(i) * j);
    /// assert_eq!(j.cross(k), Matrix3f::cross(j) * k);
    /// assert_eq!(k.cross(i), Matrix3f::cross(k) * i);
    /// ```
    pub fn cross(a: Vec3<T>) -> Matrix3<T> {
        Matrix3::new(
            T::zero(),
            -a.z(),
            a.y(),
            a.z(),
            T::zero(),
            -a.x(),
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

    /// # Examples
    ///
    /// ```rust
    /// use math::assert_eq_eps;
    /// use math::matrix::*;
    /// use math::vector::*;
    /// use std::f32::consts::{FRAC_PI_2, PI};
    ///
    ///
    /// let i = Vec3f::new(1.0, 0.0, 0.0);
    /// let j = Vec3f::new(0.0, 1.0, 0.0);
    /// let k = Vec3f::new(0.0, 0.0, 1.0);
    ///
    /// let rot: Matrix3f = Matrix3f::rotation_z(FRAC_PI_2);
    /// assert_eq_eps!(rot * i, j, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_z(-FRAC_PI_2);
    /// assert_eq_eps!(rot * i, -j, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_z(PI);
    /// assert_eq_eps!(rot * i, -i, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_z(FRAC_PI_2);
    /// assert_eq_eps!(rot * j, -i, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_z(-FRAC_PI_2);
    /// assert_eq_eps!(rot * j, i, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_z(PI);
    /// assert_eq_eps!(rot * j, -j, 0.00000001);
    ///
    /// let rot: Matrix3f = Matrix3f::rotation_x(FRAC_PI_2);
    /// assert_eq_eps!(rot * j, k, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_x(-FRAC_PI_2);
    /// assert_eq_eps!(rot * j, -k, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_x(PI);
    /// assert_eq_eps!(rot * j, -j, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_x(FRAC_PI_2);
    /// assert_eq_eps!(rot * k, -j, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_x(-FRAC_PI_2);
    /// assert_eq_eps!(rot * k, j, 0.00000001);
    /// let rot: Matrix3f = Matrix3f::rotation_x(PI);
    /// assert_eq_eps!(rot * k, -k, 0.00000001);
    /// ```
    pub fn rotation(theta: T, a: Vec3<T>) -> Matrix3<T> {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        // To rotate v about a by theta we want
        // v' = cos_theta * v + (1 - cos_theta) * (v . a) * a + sin_theta * (a x v)
        // Here we just squash what's done to v on the right hand side into a single matrix
        cos_theta * Matrix3::one()
            + (T::one() - cos_theta) * a.outer(a)
            + sin_theta * Matrix3::cross(a)
    }

    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    ///
    /// let zero: Matrix3f = Matrix3::zero();
    /// let id: Matrix3f = Matrix3::one();
    /// let ortho = Matrix3::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    ///
    /// assert_eq!(zero.inverse(), None);
    /// assert_eq!(id.inverse().unwrap(), id);
    /// assert_eq!(ortho.inverse().unwrap(), ortho.transpose());
    /// ```
    pub fn inverse(&self) -> Option<Self> {
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

impl Matrix4f {
    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    ///
    /// let zero = Matrix4f::zero();
    /// let id = Matrix4f::one();
    ///
    /// assert_eq!(zero.inverse(), None);
    /// assert_eq!(id.inverse().unwrap(), id);
    /// ```
    pub fn inverse(&self) -> Option<Self> {
        // Implementation from Foundations of Game Engine Development, Volume 1
        let a: Vec3f = self.col(0).into();
        let b: Vec3f = self.col(1).into();
        let c: Vec3f = self.col(2).into();
        let d: Vec3f = self.col(3).into();

        let x = self.get(3, 0);
        let y = self.get(3, 1);
        let z = self.get(3, 2);
        let w = self.get(3, 3);

        let mut s = a.cross(b);
        let mut t = c.cross(d);
        let mut u = a * y - b * x;
        let mut v = c * w - d * z;

        let det = s.dot(v) + t.dot(u);
        if det == 0.0 {
            return None;
        }
        let inv_det = 1.0 / det;
        s = inv_det * s;
        t = inv_det * t;
        u = inv_det * u;
        v = inv_det * v;

        let r0 = b.cross(v) + t * y;
        let r1 = v.cross(a) - t * x;
        let r2 = d.cross(u) + s * w;
        let r3 = u.cross(c) - s * z;

        Some(Self::new(
            r0.x(),
            r0.y(),
            r0.z(),
            -b.dot(t),
            r1.x(),
            r1.y(),
            r1.z(),
            a.dot(t),
            r2.x(),
            r2.y(),
            r2.z(),
            -d.dot(s),
            r3.x(),
            r3.y(),
            r3.z(),
            c.dot(s),
        ))
    }
}

impl<T: VecElem, const N: usize> Zero for Matrix<T, N> {
    fn zero() -> Matrix<T, N> {
        Matrix {
            columns: [Vector::zero(); N],
        }
    }

    fn is_zero(&self) -> bool {
        self.columns().all(|c| c.iter().all(|v| v.is_zero()))
    }
}

impl<T: VecElem, const N: usize> One for Matrix<T, N> {
    fn one() -> Self {
        let mut m = Self::zero();
        for i in 0..N {
            m.set(i, i, T::one());
        }
        m
    }
}

impl<T: VecElem + Sub<Output = T>> Matrix3<T> {
    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::one();
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
    /// use math::matrix::*;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::one();
    /// let ortho = Matrix3f::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    /// let unortho = Matrix3f::new(0.0, 2.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    ///
    /// assert!(!zero.is_orthogonal());
    /// assert!(id.is_orthogonal());
    /// assert!(ortho.is_orthogonal());
    /// assert!(!unortho.is_orthogonal());
    /// ```
    pub fn is_orthogonal(&self) -> bool {
        (*self) * self.transpose() == Matrix3::one()
    }
}

//
// Arithmetic
//

impl<T: VecElem, const N: usize> Add for Matrix<T, N> {
    type Output = Matrix<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::one();
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

impl<T: VecElem, const N: usize> Mul for Matrix<T, N> {
    type Output = Self;

    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::one();
    ///
    /// assert_eq!(zero * id, zero);
    /// assert_eq!(id * zero, zero);
    /// assert_eq!(zero * zero, zero);
    /// assert_eq!(id * id, id);
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        let mut m = Self::zero();
        for i in 0_usize..N {
            for j in 0_usize..N {
                m.set(i, j, self.row(i).dot(rhs.col(j)));
            }
        }
        m
    }
}

impl<T: VecElem, const N: usize> Mul<Vector<T, N>> for Matrix<T, N> {
    type Output = Vector<T, N>;

    /// # Examples
    ///
    /// ```rust
    /// use math::matrix::*;
    /// use math::vector::*;
    ///
    /// let zero = Matrix3f::zero();
    /// let id = Matrix3f::one();
    /// let flip = Matrix3f::new(0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0);
    /// let v = Vec3f::new(1.0, 2.0, 3.0);
    /// let v_zero = Vec3f::zero();
    ///
    /// assert_eq!(zero * v, v_zero);
    /// assert_eq!(id * v, v);
    /// assert_eq!(flip * v, Vec3f::new(3.0, 2.0, 1.0));
    /// ```
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Self::Output) -> Self::Output {
        self.columns()
            .zip(rhs.iter())
            .map(|(col, c)| (*col) * c)
            .fold(Self::Output::zero(), |acc, v| acc + v)
    }
}

impl<T: VecElem, const N: usize> Mul<T> for &Matrix<T, N> {
    type Output = Matrix<T, N>;

    fn mul(self, rhs: T) -> Self::Output {
        self.columns().map(|col| (*col) * rhs).collect()
    }
}

impl<T: VecElem, const N: usize> Mul<T> for Matrix<T, N> {
    type Output = Matrix<T, N>;

    fn mul(self, rhs: T) -> Self::Output {
        self.columns().map(|col| (*col) * rhs).collect()
    }
}

impl<const N: usize> Mul<&Matrix<f32, N>> for f32 {
    type Output = Matrix<f32, N>;

    fn mul(self, rhs: &Self::Output) -> Self::Output {
        rhs * self
    }
}

impl<const N: usize> Mul<Matrix<f32, N>> for f32 {
    type Output = Matrix<f32, N>;

    fn mul(self, rhs: Self::Output) -> Self::Output {
        rhs * self
    }
}

//
// Iterators
//

pub struct MatrixIterator<'a, T: VecElem, const N: usize> {
    iter: Iter<'a, Vector<T, N>>,
}

impl<'a, T: VecElem, const N: usize> IntoIterator for &'a Matrix<T, N> {
    type Item = &'a Vector<T, N>;
    type IntoIter = MatrixIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        MatrixIterator {
            iter: self.columns.iter(),
        }
    }
}

impl<'a, T: VecElem, const N: usize> Iterator for MatrixIterator<'a, T, N> {
    type Item = &'a Vector<T, N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct MatrixMutIterator<'a, T: VecElem, const N: usize> {
    iter: std::slice::IterMut<'a, Vector<T, N>>,
}

impl<'a, T: VecElem, const N: usize> IntoIterator for &'a mut Matrix<T, N> {
    type Item = &'a mut Vector<T, N>;
    type IntoIter = MatrixMutIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        MatrixMutIterator {
            iter: self.columns.iter_mut(),
        }
    }
}

impl<'a, T: VecElem, const N: usize> Iterator for MatrixMutIterator<'a, T, N> {
    type Item = &'a mut Vector<T, N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: VecElem, const N: usize> FromIterator<Vector<T, N>> for Matrix<T, N> {
    fn from_iter<I: IntoIterator<Item = Vector<T, N>>>(iter: I) -> Self {
        let mut v_iter = iter.into_iter().take(N);
        let mut m = Self::zero();
        for i in 0..N {
            m.columns[i] = v_iter.next().unwrap_or(Vector::zero());
        }
        m
    }
}
