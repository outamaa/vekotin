use crate::geometry::line_segment::{LineSegment, LineSegment2i};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::mem;

// TODO: Maybe add some trait like Canvas, but for now let's start with something, ie. using SDL
pub fn draw_point(canvas: &mut Canvas<Window>, x: i32, y: i32, color: Color) {
    canvas.set_draw_color(color);
    canvas.draw_point(Point::new(x, y));
}

// Bresenham's line drawing algorithm, ported from ssloy/tinyrenderer
pub fn draw_line_segment(canvas: &mut Canvas<Window>, line_segment: &LineSegment2i, color: Color) {
    let mut x0 = line_segment.start.x();
    let mut y0 = line_segment.start.y();
    let mut x1 = line_segment.end.x();
    let mut y1 = line_segment.end.y();

    let mut steep = false;
    if (x0 - x1).abs() < (y0 - y1).abs() {
        mem::swap(&mut x0, &mut y0);
        mem::swap(&mut x1, &mut y1);
        steep = true;
    }
    if x0 > x1 {
        mem::swap(&mut x0, &mut x1);
        mem::swap(&mut y0, &mut y1);
    }
    let dx = x1 - x0;
    let dy = y1 - y0;
    let d_error = (dy as f32 / dx as f32).abs();
    let mut error = 0.0f32;
    let mut y = y0;
    for x in x0..=x1 {
        if steep {
            draw_point(canvas, y as i32, x, color);
        } else {
            draw_point(canvas, x, y as i32, color);
        }
        error += d_error;
        if error > 0.5 {
            y += if y1 > y0 { 1 } else { -1 };
            error -= 1.0;
        }
    }
}
