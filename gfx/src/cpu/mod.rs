pub mod canvas;

use crate::color::Color;
use canvas::Canvas;
use geometry::line_segment::LineSegment2i;
use geometry::transform::Transform;
use geometry::triangle::{Triangle2f, Triangle3f, Triangle4f};
use geometry::{Point3f, Point4f};
use loader::obj::Obj;
use loader::png::Png;
use std::cmp;
use std::cmp::Ordering::Equal;
use std::mem;

// Bresenham's line drawing algorithm, ported from ssloy/tinyrenderer
pub fn draw_line_segment(canvas: &mut Canvas, line_segment: &LineSegment2i, color: Color) {
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
            canvas.draw_point(y, x, color);
        } else {
            canvas.draw_point(x, y, color);
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
            buf: vec![f32::MAX; (width * height) as usize],
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

fn interpolate_color_from_texture(
    texture: &Png,
    texture_triangle: &Triangle2f,
    bary: &Point3f,
) -> Color {
    let coords = texture_triangle.interpolate(bary);
    let x = (coords.x() * texture.width as f32).floor() as u32;
    let y = texture.height - (coords.y() * texture.height as f32).floor() as u32;
    if x >= texture.width || y >= texture.height {
        println!("Invalid x or y: {} {}", x, y);
        return Color::rgb(255, 0, 0);
    }
    let i = (texture.bytes_per_pixel as u32 * (texture.width * y + x)) as usize;
    Color::rgb(texture.data[i], texture.data[i + 1], texture.data[i + 2])
}

pub fn draw_triangle(
    canvas: &mut Canvas,
    triangle: &Triangle4f,
    normal_triangle: &Triangle3f,
    texture_triangle: &Triangle2f,
    texture: &Png,
    z_buffer: &mut ZBuffer,
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

    let min_x = cmp::max(0, min_x.floor() as i32 - 1);
    let min_y = cmp::max(0, min_y.floor() as i32 - 1);
    let max_x = cmp::min(z_buffer.width as i32, max_x.ceil() as i32 + 1);
    let max_y = cmp::min(z_buffer.height as i32, max_y.ceil() as i32 + 1);

    for y in min_y..max_y {
        for x in min_x..max_x {
            let x_f = x as f32;
            let y_f = y as f32;
            let p = Point3f::new(x_f, y_f, 0.0);

            match triangle.pc_barycentric_coordinates(&p) {
                None => {
                    continue;
                }
                Some(b) => {
                    if b.x() < 0.0 || b.y() < 0.0 || b.z() < 0.0 {
                        continue;
                    } else {
                        let p = triangle.interpolate(&b);
                        let n_z = normal_triangle.interpolate(&b).z();
                        let coeff = n_z * n_z;
                        let c = interpolate_color_from_texture(texture, texture_triangle, &b);
                        let c = Color::rgb(
                            (c.r as f32 * coeff) as u8,
                            (c.g as f32 * coeff) as u8,
                            (c.b as f32 * coeff) as u8,
                        );
                        if z_buffer.get(x as u32, y as u32) > p.z() {
                            z_buffer.set(x as u32, y as u32, p.z());
                            canvas.draw_point(x, y, c);
                        }
                    }
                }
            }
        }
    }
}

pub fn draw_obj(
    canvas: &mut Canvas,
    obj: &Obj,
    texture: &Png,
    view_xform: Transform,
    projection_xform: Transform,
) {
    let view_xform = projection_xform * view_xform;
    let width = canvas.width;
    let height = canvas.height;

    let mut z_buffer = ZBuffer::new(width, height);

    for i in 0..obj.vertex_index_triples.len() {
        let v_indices = &obj.vertex_index_triples[i];
        let t_indices = &obj.uv_index_triples[i];
        let n_indices = &obj.normal_index_triples[i];

        let v0 = view_xform * Point3f::from(obj.vertices[v_indices.0 as usize]);
        let v0 = v0.perspective_divide();
        // Project the 3D points onto the canvas, orthographic projection
        let p0 = Point4f::new(
            (v0.x() + 1.0) * width as f32 / 2.0,
            height as f32 - ((v0.y() + 1.0) * height as f32 / 2.0),
            v0.z(),
            v0.w(),
        );
        let v1 = view_xform * Point3f::from(obj.vertices[v_indices.1 as usize]);
        let v1 = v1.perspective_divide();
        let p1 = Point4f::new(
            (v1.x() + 1.0) * width as f32 / 2.0,
            height as f32 - ((v1.y() + 1.0) * height as f32 / 2.0),
            v1.z(),
            v1.w(),
        );
        let v2 = view_xform * Point3f::from(obj.vertices[v_indices.2 as usize]);
        let v2 = v2.perspective_divide();
        let p2 = Point4f::new(
            (v2.x() + 1.0) * width as f32 / 2.0,
            height as f32 - ((v2.y() + 1.0) * height as f32 / 2.0),
            v2.z(),
            v2.w(),
        );

        let f = Triangle4f::new(&p0, &p1, &p2);

        if f.normal().z() <= 0.0 {
            let n0 = Point3f::from(view_xform * obj.normals[n_indices.0 as usize]);
            let n1 = Point3f::from(view_xform * obj.normals[n_indices.1 as usize]);
            let n2 = Point3f::from(view_xform * obj.normals[n_indices.2 as usize]);
            let n = Triangle3f::new(&n0, &n1, &n2);

            let t0 = obj.uvs[t_indices.0 as usize].into();
            let t1 = obj.uvs[t_indices.1 as usize].into();
            let t2 = obj.uvs[t_indices.2 as usize].into();
            let t = Triangle2f::new(&t0, &t1, &t2);
            draw_triangle(canvas, &f, &n, &t, texture, &mut z_buffer);
        }
    }
}
