use crate::geometry::point::Point;
use crate::math::vector::VecElem;

pub struct LineSegment<T: VecElem, const N: usize> {
    pub start: Point<T, N>,
    pub end: Point<T, N>,
}

impl<T: VecElem, const N: usize> LineSegment<T, N> {
    pub fn new(start: Point<T, N>, end: Point<T, N>) -> LineSegment<T, N> {
        LineSegment { start, end }
    }
}

pub type LineSegment2<T> = LineSegment<T, 2>;
pub type LineSegment2f = LineSegment2<f32>;
pub type LineSegment2i = LineSegment2<i32>;

pub type LineSegment3<T> = LineSegment<T, 3>;
pub type LineSegment3f = LineSegment3<f32>;
pub type LineSegment3i = LineSegment3<i32>;
