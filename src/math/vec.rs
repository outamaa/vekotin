use std::iter::{Copied, FromIterator};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
use std::slice::Iter;

// General note: Use Copy, pass by value, trust the compiler to optimize. :)

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3D {
    components: [f32; 3],
}

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

impl Vec3D {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3D {
        Vec3D {
            components: [x, y, z],
        }
    }

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
