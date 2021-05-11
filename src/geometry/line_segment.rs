use crate::geometry::point::Point;
use crate::math::vector::VecElem;

pub struct LineSegment<'a, T: VecElem, const N: usize> {
    pub start: &'a Point<T, N>,
    pub end: &'a Point<T, N>,
}

impl<'a, T: VecElem, const N: usize> LineSegment<'a, T, N> {
    pub fn new(start: &'a Point<T, N>, end: &'a Point<T, N>) -> LineSegment<'a, T, N> {
        LineSegment { start, end }
    }
}

pub type LineSegment2<'a, T> = LineSegment<'a, T, 2>;
pub type LineSegment2f<'a> = LineSegment2<'a, f32>;
pub type LineSegment2i<'a> = LineSegment2<'a, i32>;

pub type LineSegment3<'a, T> = LineSegment<'a, T, 3>;
pub type LineSegment3f<'a> = LineSegment3<'a, f32>;
pub type LineSegment3i<'a> = LineSegment3<'a, i32>;
