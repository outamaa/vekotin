use crate::geometry::line_segment::LineSegment2i;
use crate::geometry::triangle::{Triangle2i, Triangle3f};
use crate::geometry::{Point2i, Point3f};
use crate::loader::obj::Obj;
use crate::math::Matrix3f;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::cmp;
use std::cmp::Ordering::Equal;
use std::mem;

// TODO: Maybe add some trait like Canvas, but for now let's start with something, ie. using SDL
pub fn draw_point(canvas: &mut Canvas<Window>, x: i32, y: i32, color: Color) {
    canvas.set_draw_color(color);
    canvas.draw_point(Point::new(x, y)).expect("draw_point");
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

pub struct ZBuffer {
    buf: Vec<f32>,
    width: u32,
    height: u32,
}

impl ZBuffer {
    fn new(width: u32, height: u32) -> Self {
        ZBuffer {
            buf: vec![f32::MIN; (width * height) as usize],
            width,
            height,
        }
    }

    fn set(&mut self, x: u32, y: u32, z: f32) {
        assert!(x < self.width && y < self.height);
        self.buf[(y * self.width + x) as usize] = z;
    }

    fn get(&self, x: u32, y: u32) -> f32 {
        assert!(x < self.width && y < self.height);
        self.buf[(y * self.width + x) as usize]
    }
}

pub fn draw_triangle(
    canvas: &mut Canvas<Window>,
    triangle: &Triangle3f,
    z_buffer: &mut ZBuffer,
    color: Color,
) {
    let min_x = triangle
        .points
        .iter()
        .map(|&p| p.x())
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let min_y = triangle
        .points
        .iter()
        .map(|&p| p.y())
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let max_x = triangle
        .points
        .iter()
        .map(|&p| p.x())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();
    let max_y = triangle
        .points
        .iter()
        .map(|&p| p.y())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Equal))
        .unwrap();

    let min_x = cmp::max(0, min_x.floor() as i32);
    let min_y = cmp::max(0, min_y.floor() as i32);
    let max_x = cmp::min(z_buffer.width as i32, max_x.ceil() as i32);
    let max_y = cmp::min(z_buffer.height as i32, max_y.ceil() as i32);

    for y in min_y..max_y {
        for x in min_x..max_x {
            let x_f = x as f32;
            let y_f = y as f32;
            let mut p = Point3f::new(x_f, y_f, 0.0);

            let mut p_z = 0.0;
            match triangle.barycentric_coordinates(&p) {
                None => {
                    continue;
                }
                Some(b) => {
                    if b.x() < 0.0 || b.y() < 0.0 || b.z() < 0.0 {
                        continue;
                    } else {
                        for i in 0..3 {
                            p_z += triangle.points[i].z() * b[i];
                        }
                    }
                }
            }

            if z_buffer.get(x as u32, y as u32) < p_z {
                z_buffer.set(x as u32, y as u32, p_z);
                draw_point(canvas, x, y, color);
            }
        }
    }
}

pub fn draw_obj(canvas: &mut Canvas<Window>, obj: &Obj, xform: &Matrix3f) {
    let viewport = canvas.viewport();
    let w = viewport.width();
    let h = viewport.height();

    let mut z_buffer = ZBuffer::new(w, h);

    for i in 0..obj.vertex_index_triples.len() {
        let v_indices = &obj.vertex_index_triples[i];

        let v0 = *xform * obj.vertices[v_indices.0 as usize];
        // Project the 3D points onto the canvas, orthographic projection
        let p0 = Point3f::new(
            (v0.x() + 1.0) * w as f32 / 2.0,
            h as f32 - ((v0.y() + 1.0) * h as f32 / 2.0),
            v0.z(),
        );
        let v1 = *xform * obj.vertices[v_indices.1 as usize];
        let p1 = Point3f::new(
            (v1.x() + 1.0) * w as f32 / 2.0,
            h as f32 - ((v1.y() + 1.0) * h as f32 / 2.0),
            v1.z(),
        );
        let v2 = *xform * obj.vertices[v_indices.2 as usize];
        let p2 = Point3f::new(
            (v2.x() + 1.0) * w as f32 / 2.0,
            h as f32 - ((v2.y() + 1.0) * h as f32 / 2.0),
            v2.z(),
        );
        let normal = Triangle3f::new(&v0.into(), &v1.into(), &v2.into())
            .normal()
            .unit();

        let t = Triangle3f::new(&p0, &p1, &p2);

        let white = Color::RGBA(255, 255, 255, 255);

        if normal.z() >= 0.0 {
            let intensity = (normal.z() * 255.0) as u8;
            let c = Color::RGBA(intensity, intensity, intensity, 255);
            draw_triangle(canvas, &t, &mut z_buffer, c);
            // gfx::cpu::draw_line_segment(canvas, &LineSegment2i::new(&p0, &p1), white);
            // gfx::cpu::draw_line_segment(canvas, &LineSegment2i::new(&p1, &p2), white);
            // gfx::cpu::draw_line_segment(canvas, &LineSegment2i::new(&p2, &p0), white);
        }
    }
}
