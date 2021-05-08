use crate::geometry::point::Point;
use crate::geometry::{Point2, Point2f, Point3, Point3f};
use crate::math::vector::VecElem;
use crate::math::Vec3;
use num::Float;

pub struct Triangle<T: VecElem, const N: usize> {
    pub points: [Point<T, N>; 3],
}

impl<T: VecElem, const N: usize> Triangle<T, N> {
    pub fn new(p0: &Point<T, N>, p1: &Point<T, N>, p2: &Point<T, N>) -> Triangle<T, N> {
        Triangle {
            points: [p0.clone(), p1.clone(), p2.clone()],
        }
    }
}

impl<T: VecElem> Triangle<T, 3> {
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::geometry::triangle::*;
    /// use vekotin::geometry::Point3i;
    /// use vekotin::math::Vec3i;
    ///
    /// let triangle = Triangle::new(
    ///   &Point3i::new(0, 0, 0),
    ///   &Point3i::new(1, 0, 0),
    ///   &Point3i::new(0, 2, 0)
    /// );
    ///
    /// assert_eq!(triangle.normal(), Vec3i::new(0, 0, 2));
    /// ```
    pub fn normal(&self) -> Vec3<T> {
        (self.points[1] - self.points[0]).cross(self.points[2] - self.points[0])
    }
}

impl<T: VecElem> Triangle<T, 2> {
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::geometry::triangle::*;
    /// use vekotin::geometry::Point2i;
    ///
    /// let triangle = Triangle::new(
    ///   &Point2i::new(0, 0),
    ///   &Point2i::new(0, 2),
    ///   &Point2i::new(1, 0)
    /// );
    ///
    /// assert_eq!(triangle.signed_area_doubled(), 2);
    ///
    /// let triangle = Triangle::new(
    ///   &Point2i::new(0, 0),
    ///   &Point2i::new(1, 0),
    ///   &Point2i::new(0, 2)
    /// );
    ///
    /// assert_eq!(triangle.signed_area_doubled(), -2);
    /// ```
    pub fn signed_area_doubled(&self) -> T {
        ((self.points[2].x() - self.points[0].x()) * (self.points[1].y() - self.points[0].y())
            - (self.points[1].x() - self.points[0].x()) * (self.points[2].y() - self.points[0].y()))
    }
}

impl<T: VecElem + PartialOrd> Triangle<T, 2> {
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::geometry::triangle::*;
    /// use vekotin::geometry::{Point2i, Point3f};
    /// use vekotin::math::Vec3i;
    ///
    /// let p0 = Point2i::new(0, 0);
    /// let p1 = Point2i::new(2, 0);
    /// let p2 = Point2i::new(0, 2);
    ///
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert_eq!(triangle.barycentric_coordinates(&p0), Some(Point3f::new(1.0, 0.0, 0.0)));
    /// assert_eq!(triangle.barycentric_coordinates(&p1), Some(Point3f::new(0.0, 1.0, 0.0)));
    /// assert_eq!(triangle.barycentric_coordinates(&p2), Some(Point3f::new(0.0, 0.0, 1.0)));
    /// ```
    pub fn barycentric_coordinates(&self, p: &Point2<T>) -> Option<Point3f> {
        let a2 = self.signed_area_doubled();
        if a2 == T::zero() {
            return None;
        }
        let a2 = a2.as_f32();
        let x0 = self.points[0].x().as_f32();
        let x1 = self.points[1].x().as_f32();
        let x2 = self.points[2].x().as_f32();
        let y0 = self.points[0].y().as_f32();
        let y1 = self.points[1].y().as_f32();
        let y2 = self.points[2].y().as_f32();
        let p_x = p.x().as_f32();
        let p_y = p.y().as_f32();

        let u = ((y0 - y2) / a2) * p_x + ((x2 - x0) / a2) * p_y + (x0 * y2 - x2 * y0) / a2;
        let v = ((y1 - y0) / a2) * p_x + ((x0 - x1) / a2) * p_y + (x1 * y0 - x0 * y1) / a2;

        Some(Point3::new(1.0 - u - v, u, v))
    }

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::geometry::triangle::*;
    /// use vekotin::geometry::{Point2i, Point3f};
    /// use vekotin::math::Vec3i;
    ///
    /// let p0 = Point2i::new(0, 0);
    /// let p1 = Point2i::new(2, 0);
    /// let p2 = Point2i::new(0, 2);
    ///
    /// let p_outside = Point2i::new(-1, -1);
    /// let p_ones = Point2i::new(1, 1);
    ///
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert!(triangle.contains(&p0));
    /// assert!(triangle.contains(&p1));
    /// assert!(triangle.contains(&p2));
    /// assert!(triangle.contains(&p_ones));
    /// assert!(!triangle.contains(&p_outside));
    /// ```
    pub fn contains(&self, p: &Point2<T>) -> bool {
        let bary = self.barycentric_coordinates(p);
        match bary {
            None => false,
            Some(p) => p.x() >= 0.0 && p.y() >= 0.0 && p.z() >= 0.0,
        }
    }
}

impl<T: VecElem + Ord> Triangle<T, 2> {
    pub fn bounding_box(&self) -> (Point2<T>, Point2<T>) {
        let min_x = self.points.iter().map(|&p| p.x()).min().unwrap();
        let min_y = self.points.iter().map(|&p| p.y()).min().unwrap();
        let max_x = self.points.iter().map(|&p| p.x()).max().unwrap();
        let max_y = self.points.iter().map(|&p| p.y()).max().unwrap();

        (Point2::new(min_x, min_y), Point2::new(max_x, max_y))
    }
}

pub type Triangle2<T> = Triangle<T, 2>;
pub type Triangle2f = Triangle2<f32>;
pub type Triangle2i = Triangle2<i32>;

pub type Triangle3<T> = Triangle<T, 3>;
pub type Triangle3f = Triangle3<f32>;
pub type Triangle3i = Triangle3<i32>;
