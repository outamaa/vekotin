use crate::point::Point;
use crate::{Point2, Point2f, Point3, Point3f, Point4f};
use math::vector::{VecElem, Zero};
use math::{Vec3, Vec3f, Vec4f, Vector};

#[derive(Debug)]
pub struct Triangle<'a, T: VecElem, const N: usize> {
    pub points: [&'a Point<T, N>; 3],
}

impl<'a, T: VecElem, const N: usize> Triangle<'a, T, N> {
    pub fn new(
        p0: &'a Point<T, N>,
        p1: &'a Point<T, N>,
        p2: &'a Point<T, N>,
    ) -> Triangle<'a, T, N> {
        Triangle {
            points: [p0, p1, p2],
        }
    }
}

impl<'a, T: VecElem> Triangle<'a, T, 3> {
    /// # Examples
    ///
    /// ```rust
    /// use geometry::triangle::*;
    /// use geometry::Point3i;
    /// use math::Vec3i;
    ///
    /// let p0 = Point3i::new(0, 0, 0);
    /// let p1 = Point3i::new(1, 0, 0);
    /// let p2 = Point3i::new(0, 2, 0);
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert_eq!(triangle.normal(), Vec3i::new(0, 0, 2));
    /// ```
    pub fn normal(&self) -> Vec3<T> {
        (*self.points[1] - *self.points[0]).cross(*self.points[2] - *self.points[0])
    }
}

impl<'a, T: VecElem> Triangle<'a, T, 2> {
    /// # Examples
    ///
    /// ```rust
    /// use geometry::triangle::*;
    /// use geometry::Point2i;
    /// let p0 = Point2i::new(0, 0);
    /// let p1 = Point2i::new(0, 2);
    /// let p2 = Point2i::new(1, 0);
    ///
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert_eq!(triangle.signed_area_doubled(), 2);
    ///
    /// let p0 = Point2i::new(0, 0);
    /// let p1 = Point2i::new(1, 0);
    /// let p2 = Point2i::new(0, 2);
    ///
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert_eq!(triangle.signed_area_doubled(), -2);
    /// ```
    pub fn signed_area_doubled(&self) -> T {
        (self.points[2].x() - self.points[0].x()) * (self.points[1].y() - self.points[0].y())
            - (self.points[1].x() - self.points[0].x()) * (self.points[2].y() - self.points[0].y())
    }
}

impl<'a, T: VecElem + PartialOrd> Triangle<'a, T, 2> {
    /// # Examples
    ///
    /// ```rust
    /// use geometry::triangle::*;
    /// use geometry::{Point2i, Point3f};
    /// use math::Vec3i;
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
        let x0 = self.points[0].x();
        let x1 = self.points[1].x();
        let x2 = self.points[2].x();
        let y0 = self.points[0].y();
        let y1 = self.points[1].y();
        let y2 = self.points[2].y();
        let p_x = p.x();
        let p_y = p.y();

        // Work as long as possible without casting to f32
        let u = ((y0 - y2) * p_x + (x2 - x0) * p_y + (x0 * y2 - x2 * y0)).as_f32();
        let v = ((y1 - y0) * p_x + (x0 - x1) * p_y + (x1 * y0 - x0 * y1)).as_f32();

        Some(Point3::new(1.0 - (u + v) / a2, u / a2, v / a2))
    }

    /// # Examples
    ///
    /// ```rust
    /// use geometry::triangle::*;
    /// use geometry::{Point2i, Point3f};
    /// use math::Vec3i;
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

    pub fn interpolate(&self, bary: &Point3f) -> Point2f {
        let mut v = Vector::zero();
        for i in 0..3 {
            v = v + self.points[i].as_vector().as_f32() * bary[i];
        }
        v.into()
    }
}

impl<'a, T: VecElem + PartialOrd> Triangle<'a, T, 3> {
    /// # Examples
    ///
    /// ```rust
    /// use geometry::triangle::*;
    /// use geometry::{Point3i, Point3f};
    /// use math::Vec3i;
    ///
    /// let p0 = Point3i::new(0, 0, 0);
    /// let p1 = Point3i::new(2, 0, 0);
    /// let p2 = Point3i::new(0, 2, 0);
    ///
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert_eq!(triangle.barycentric_coordinates(&p0), Some(Point3f::new(1.0, 0.0, 0.0)));
    /// assert_eq!(triangle.barycentric_coordinates(&p1), Some(Point3f::new(0.0, 1.0, 0.0)));
    /// assert_eq!(triangle.barycentric_coordinates(&p2), Some(Point3f::new(0.0, 0.0, 1.0)));
    /// ```
    pub fn barycentric_coordinates(&self, p: &Point3<T>) -> Option<Point3f> {
        // TODO this works for orthographic projection, need to implement perspective correct coordinates for perspective projection
        let n = self.normal();
        let a2 = n.length_squared();

        if a2.as_f32() < 0.0001 {
            // Degenerate triangle
            return None;
        }

        let p0 = *self.points[0];
        let p1 = *self.points[1];
        let p2 = *self.points[2];

        let u = (p2 - p0).cross(p0 - *p).dot(n);
        let v = (p0 - *p).cross(p1 - p0).dot(n);

        Some(Point3::new(
            (T::one() - (u + v) / a2).as_f32(),
            (u / a2).as_f32(),
            (v / a2).as_f32(),
        ))
    }

    pub fn contains(&self, p: &Point3<T>) -> bool {
        let bary = self.barycentric_coordinates(p);
        match bary {
            None => false,
            Some(p) => p.x() >= 0.0 && p.y() >= 0.0 && p.z() >= 0.0,
        }
    }

    pub fn interpolate(&self, bary: &Point3f) -> Point3f {
        let mut v = Vector::zero();
        for i in 0..3 {
            v += self.points[i].as_vector().as_f32() * bary[i];
        }
        v.into()
    }
}

impl<'a> Triangle4f<'a> {
    /// # Examples
    ///
    /// ```rust
    /// use geometry::triangle::*;
    /// use geometry::{Point3f, Point4f};
    /// use math::Vec3i;
    ///
    /// let p0 = Point4f::new(0., 0., 0., 1.);
    /// let p1 = Point4f::new(2., 0., 0., 1.);
    /// let p2 = Point4f::new(0., 2., 0., 1.);
    ///
    /// let triangle = Triangle::new(&p0, &p1, &p2);
    ///
    /// assert_eq!(triangle.barycentric_coordinates(&p0), Some(Point3f::new(1.0, 0.0, 0.0)));
    /// assert_eq!(triangle.barycentric_coordinates(&p1), Some(Point3f::new(0.0, 1.0, 0.0)));
    /// assert_eq!(triangle.barycentric_coordinates(&p2), Some(Point3f::new(0.0, 0.0, 1.0)));
    /// ```
    pub fn barycentric_coordinates(&self, p: &Point3f) -> Option<Point3f> {
        let p0 = self.points[0].xyz();
        let p1 = self.points[1].xyz();
        let p2 = self.points[2].xyz();
        let tri3f = Triangle::new(&p0, &p1, &p2);
        tri3f.barycentric_coordinates(p)
    }

    /// Calculate perspective corrected barycentric coordinates
    pub fn pc_barycentric_coordinates(&self, p: &Point3f) -> Option<Point3f> {
        self.barycentric_coordinates(p).map(|bc| {
            let u = bc.y();
            let v = bc.z();

            let w0 = self.points[0].w();
            let w1 = self.points[1].w();
            let w2 = self.points[2].w();

            let w_inv = (1.0 - u - v) / w0 + u / w1 + v / w2;

            let w = 1.0 / w_inv;

            let u_pc = (w * u) / w1;
            let v_pc = (w * v) / w2;

            Point3f::new(1.0 - u_pc - v_pc, u_pc, v_pc)
        })
    }

    pub fn contains(&self, p: &Point3f) -> bool {
        let bary = self.barycentric_coordinates(p);
        match bary {
            None => false,
            Some(p) => p.x() >= 0.0 && p.y() >= 0.0 && p.z() >= 0.0,
        }
    }

    pub fn interpolate(&self, bary: &Point3f) -> Point4f {
        let mut v = Vec4f::zero();
        for i in 0..3 {
            v = v + *self.points[i].as_vector() * bary[i];
        }
        v.into()
    }

    /// Calculate normal as if the points were 3D
    pub fn normal(&self) -> Vec3f {
        (self.points[1].as_vector().xyz() - self.points[0].as_vector().xyz())
            .cross(self.points[2].as_vector().xyz() - self.points[0].as_vector().xyz())
    }
}

pub type Triangle2<'a, T> = Triangle<'a, T, 2>;
pub type Triangle2f<'a> = Triangle2<'a, f32>;
pub type Triangle2i<'a> = Triangle2<'a, i32>;

pub type Triangle3<'a, T> = Triangle<'a, T, 3>;
pub type Triangle3f<'a> = Triangle3<'a, f32>;
pub type Triangle3i<'a> = Triangle3<'a, i32>;

pub type Triangle4<'a, T> = Triangle<'a, T, 4>;
pub type Triangle4f<'a> = Triangle4<'a, f32>;
