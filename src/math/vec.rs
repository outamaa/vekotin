use std::iter::{Copied, FromIterator};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
use std::slice::Iter;

// General note: Use Copy, pass by value, trust the compiler to optimize. :)
// Iterators used heavily to help with copy paste / macrology for dimensions other than 3

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3D {
    components: [f32; 3],
}

impl Vec3D {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3D {
        Vec3D {
            components: [x, y, z],
        }
    }

    /// Returns a 3D vector with all components set as zeroes
    pub fn zero() -> Vec3D {
        Vec3D::new(0.0, 0.0, 0.0)
    }

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    /// let mut v_iter = v.iter();
    /// assert_eq!(1.0, v_iter.next().unwrap());
    /// assert_eq!(2.0, v_iter.next().unwrap());
    /// assert_eq!(3.0, v_iter.next().unwrap());
    /// assert!(v_iter.next().is_none());
    ///
    /// let v2 = v.iter().collect();
    /// assert_eq!(v, v2);
    /// ```
    pub fn iter(&self) -> Vec3DIterator {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> Vec3DMutIterator {
        self.into_iter()
    }

    pub fn x(&self) -> f32 {
        self.components[0]
    }
    pub fn y(&self) -> f32 {
        self.components[1]
    }
    pub fn z(&self) -> f32 {
        self.components[2]
    }

    /// Returns the dot - or inner - product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let i = Vec3D::new(1.0, 0.0, 0.0);
    /// let j = Vec3D::new(0.0, 1.0, 0.0);
    /// let k = Vec3D::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(1.0, i.dot(i));
    /// assert_eq!(0.0, j.dot(i));
    /// assert_eq!(0.0, i.dot(j));
    /// assert_eq!(1.0, k.dot(k));
    /// ```
    pub fn dot(&self, other: Vec3D) -> f32 {
        let mut sum = 0.0;
        for (c_self, c_other) in self.iter().zip(other.iter()) {
            sum += c_self * c_other;
        }
        sum
    }

    /// Returns the cross product of `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3D::zero();
    /// let i = Vec3D::new(1.0, 0.0, 0.0);
    /// let j = Vec3D::new(0.0, 1.0, 0.0);
    /// let k = Vec3D::new(0.0, 0.0, 1.0);
    ///
    /// assert_eq!(zero, i.cross(i));
    /// assert_eq!(i, j.cross(k));
    /// assert_eq!(j, k.cross(i));
    /// assert_eq!(k, i.cross(j));
    /// ```
    pub fn cross(&self, other: Vec3D) -> Vec3D {
        Vec3D::new(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }
}

//
// Arithmetic
//

impl Add for Vec3D {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3D::zero();
    /// let i = Vec3D::new(1.0, 0.0, 0.0);
    /// let j = Vec3D::new(0.0, 1.0, 0.0);
    /// let k = Vec3D::new(0.0, 0.0, 1.0);
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

impl AddAssign for Vec3D {
    fn add_assign(&mut self, rhs: Self) {
        for (i, c) in self.iter_mut().enumerate() {
            *c += rhs[i];
        }
    }
}

impl Sub for Vec3D {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3D::zero();
    /// let i = Vec3D::new(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(i - zero, i);
    /// assert_eq!(i - i , zero);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        self.iter().zip(rhs.iter()).map(|(a, b)| a - b).collect()
    }
}

impl SubAssign for Vec3D {
    fn sub_assign(&mut self, rhs: Self) {
        for (i, c) in self.iter_mut().enumerate() {
            *c -= rhs[i];
        }
    }
}

impl Neg for Vec3D {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let zero = Vec3D::zero();
    /// let i = Vec3D::new(1.0, 0.0, 0.0);
    ///
    /// assert_eq!(-zero, zero);
    /// assert_eq!(-i, Vec3D::new(-1.0, 0.0, 0.0));
    /// ```
    fn neg(self) -> Self::Output {
        self.iter().map(|a| -a).collect()
    }
}

impl Mul<f32> for Vec3D {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(v * 2.0, Vec3D::new(2.0, 4.0, 6.0));
    /// ```
    fn mul(self, rhs: f32) -> Self::Output {
        self.iter().map(|a| rhs * a).collect()
    }
}

impl Mul<Vec3D> for f32 {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(2.0 * v, Vec3D::new(2.0, 4.0, 6.0));
    /// ```
    fn mul(self, rhs: Vec3D) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vec3D {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(v / 2.0, Vec3D::new(0.5, 1.0, 1.5));
    /// ```
    fn div(self, rhs: f32) -> Self::Output {
        let inv_rhs = 1.0 / rhs;
        self.iter().map(|a| a * inv_rhs).collect()
    }
}

impl Div<Vec3D> for f32 {
    type Output = Vec3D;

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    ///
    /// assert_eq!(6.0 / v, Vec3D::new(6.0, 3.0, 2.0));
    /// ```
    fn div(self, rhs: Vec3D) -> Self::Output {
        rhs.iter().map(|a| self / a).collect()
    }
}

//
// Iterators
//

pub struct Vec3DIterator<'a> {
    iter: Copied<Iter<'a, f32>>,
}

impl<'a> IntoIterator for &'a Vec3D {
    type Item = f32;
    type IntoIter = Vec3DIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Vec3DIterator {
            iter: self.components.iter().copied(),
        }
    }
}

impl<'a> Iterator for Vec3DIterator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct Vec3DMutIterator<'a> {
    iter: ::std::slice::IterMut<'a, f32>,
}

impl<'a> IntoIterator for &'a mut Vec3D {
    type Item = &'a mut f32;
    type IntoIter = Vec3DMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Vec3DMutIterator {
            iter: self.components.iter_mut(),
        }
    }
}

impl<'a> Iterator for Vec3DMutIterator<'a> {
    type Item = &'a mut f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl FromIterator<f32> for Vec3D {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        let mut v_iter = iter.into_iter().take(3);
        let x = v_iter.next().unwrap_or(0.0);
        let y = v_iter.next().unwrap_or(0.0);
        let z = v_iter.next().unwrap_or(0.0);
        Vec3D {
            components: [x, y, z],
        }
    }
}

//
// Indexing
//

impl Index<usize> for Vec3D {
    type Output = f32;

    /// # Examples
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let v = Vec3D::new(1.0, 2.0, 3.0);
    /// assert_eq!(v[0], 1.0);
    /// assert_eq!(v[1], 2.0);
    /// assert_eq!(v[2], 3.0);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.components[index]
    }
}

impl IndexMut<usize> for Vec3D {
    /// # Examples
    /// ```rust
    /// use vekotin::math::vec::*;
    ///
    /// let mut v = Vec3D::new(1.0, 2.0, 3.0);
    /// v[0] = 2.0;
    /// assert_eq!(v[0], 2.0);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}
