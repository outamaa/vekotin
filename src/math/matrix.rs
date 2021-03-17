use crate::math::vec::Vec3D;
use std::iter::FromIterator;
use std::ops::{Add, Mul};
use std::slice::Iter;

// Note: COLUMN major data layout, but usual row major indexing with get

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix3D {
    columns: [Vec3D; 3],
}

impl Matrix3D {
    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Matrix3D {
        Matrix3D {
            columns: [
                Vec3D::new(m00, m10, m20),
                Vec3D::new(m01, m11, m21),
                Vec3D::new(m02, m12, m22),
            ],
        }
    }

    pub fn from_columns(x: Vec3D, y: Vec3D, z: Vec3D) -> Matrix3D {
        Matrix3D { columns: [x, y, z] }
    }

    pub fn zero() -> Matrix3D {
        Matrix3D::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub fn identity() -> Matrix3D {
        Matrix3D::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.columns[col][row]
    }
    pub fn set(&mut self, row: usize, col: usize, val: f32) -> &mut Self {
        self.columns[col][row] = val;
        self
    }

    pub fn row(&self, row: usize) -> Vec3D {
        Vec3D::new(self.get(row, 0), self.get(row, 1), self.get(row, 2))
    }

    pub fn col(&self, col: usize) -> Vec3D {
        self.columns[col]
    }

    pub fn columns(&self) -> Matrix3DIterator {
        self.into_iter()
    }

    pub fn columns_mut(&mut self) -> Matrix3DMutIterator {
        self.into_iter()
    }
}

//
// Arithmetic
//

impl Add for Matrix3D {
    type Output = Matrix3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3D;
    ///
    /// let zero = Matrix3D::zero();
    /// let id = Matrix3D::identity();
    ///
    /// assert_eq!(zero + id, id);
    /// assert_eq!(id + zero, id);
    /// assert_eq!(zero + zero, zero);
    /// assert_eq!(id + id, Matrix3D::new(2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0));
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        self.columns()
            .zip(rhs.columns())
            .map(|(a, b)| *a + *b)
            .collect()
    }
}

impl Mul for Matrix3D {
    type Output = Matrix3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3D;
    ///
    /// let zero = Matrix3D::zero();
    /// let id = Matrix3D::identity();
    ///
    /// assert_eq!(zero * id, zero);
    /// assert_eq!(id * zero, zero);
    /// assert_eq!(zero * zero, zero);
    /// assert_eq!(id * id, id);
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        let mut m = Matrix3D::zero();
        for i in 0 as usize..3 {
            for j in 0 as usize..3 {
                m.set(i, j, self.row(i).dot(rhs.col(j)));
            }
        }
        m
    }
}

impl Mul<Vec3D> for Matrix3D {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::matrix::Matrix3D;
    /// use vekotin::math::vec::Vec3D;
    ///
    /// let zero = Matrix3D::zero();
    /// let id = Matrix3D::identity();
    /// let flip = Matrix3D::new(0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0);
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    /// let v_zero = Vec3D::zero();
    ///
    /// assert_eq!(zero * v, v_zero);
    /// assert_eq!(id * v, v);
    /// assert_eq!(flip * v, Vec3D::new(3.0, 2.0, 1.0));
    /// ```
    fn mul(self, rhs: Vec3D) -> Self::Output {
        self.columns()
            .zip(rhs.iter())
            .map(|(col, c)| (*col) * c)
            .fold(Vec3D::zero(), |acc, v| acc + v)
    }
}

//
// Iterators
//

pub struct Matrix3DIterator<'a> {
    iter: Iter<'a, Vec3D>,
}

impl<'a> IntoIterator for &'a Matrix3D {
    type Item = &'a Vec3D;
    type IntoIter = Matrix3DIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Matrix3DIterator {
            iter: self.columns.iter(),
        }
    }
}

impl<'a> Iterator for Matrix3DIterator<'a> {
    type Item = &'a Vec3D;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct Matrix3DMutIterator<'a> {
    iter: ::std::slice::IterMut<'a, Vec3D>,
}

impl<'a> IntoIterator for &'a mut Matrix3D {
    type Item = &'a mut Vec3D;
    type IntoIter = Matrix3DMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Matrix3DMutIterator {
            iter: self.columns.iter_mut(),
        }
    }
}

impl<'a> Iterator for Matrix3DMutIterator<'a> {
    type Item = &'a mut Vec3D;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl FromIterator<Vec3D> for Matrix3D {
    fn from_iter<T: IntoIterator<Item = Vec3D>>(iter: T) -> Self {
        let mut v_iter = iter.into_iter().take(3);
        let x = v_iter.next().unwrap_or(Vec3D::zero());
        let y = v_iter.next().unwrap_or(Vec3D::zero());
        let z = v_iter.next().unwrap_or(Vec3D::zero());
        Matrix3D { columns: [x, y, z] }
    }
}
