use crate::math::{Vec2f, Vec3f};
use anyhow::{anyhow, bail, Result};
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::str::SplitWhitespace;

//
// TODO: For now, no support for e.g. 4D coordinates for vertices or 3D textures
//

//
// Public interface
//
#[derive(PartialEq, Debug)]
pub struct Obj {
    pub vertices: Vec<Vec3f>,
    pub uvs: Vec<Vec2f>,
    pub normals: Vec<Vec3f>,
    pub vertex_indices: Vec<u32>,
    pub uv_indices: Vec<u32>,
    pub normal_indices: Vec<u32>,
}

impl Obj {
    pub fn new() -> Self {
        Obj {
            vertices: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            vertex_indices: Vec::new(),
            uv_indices: Vec::new(),
            normal_indices: Vec::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = File::open(path)?;
        let mut obj = Obj::new();

        // lazy_static! {
        //     static ref v_match: Regex = Regex::new(r"^v").unwrap();
        //     static ref v_capt: Regex = Regex::new(r"^v (\d+.\d+) (\d+.\d+) (\d+.\d+)").unwrap();
        // }
        for (line_num, maybe_line) in io::BufReader::new(f).lines().enumerate() {
            let line = maybe_line?;
            let mut elems = line.split_whitespace();
            let line_type = elems.next().ok_or(anyhow!("No line type"))?;
            match line_type {
                "v" => {
                    obj.vertices.push(parse_vec3f(elems)?);
                }
                "vt" => {
                    obj.uvs.push(parse_vec2f(elems)?);
                }
                "vn" => {
                    obj.vertices.push(parse_vec3f(elems)?);
                }

                _ => {}
            }
        }
        Ok(obj)
    }
}

fn parse_vec2f<'a, T: Iterator<Item = &'a str>>(mut elements: T) -> Result<Vec2f> {
    let x = elements
        .next()
        .ok_or(anyhow!("x not found"))?
        .parse::<f32>()?;
    let y = elements
        .next()
        .ok_or(anyhow!("y not found"))?
        .parse::<f32>()?;
    Ok(Vec2f::new(x, y))
}

fn parse_vec3f<'a, T: Iterator<Item = &'a str>>(mut elements: T) -> Result<Vec3f> {
    let x = elements
        .next()
        .ok_or(anyhow!("x not found"))?
        .parse::<f32>()?;
    let y = elements
        .next()
        .ok_or(anyhow!("y not found"))?
        .parse::<f32>()?;
    let z = elements
        .next()
        .ok_or(anyhow!("z not found"))?
        .parse::<f32>()?;
    Ok(Vec3f::new(x, y, z))
}

type FaceIndexTriple = (u32, u32, u32);

fn parse_face<'a, T: Iterator<Item = &'a str>>(
    mut elements: T, // ["1/2/3", "2/3/4", ...]
) -> Result<Vec<FaceIndexTriple>> {
    let triples = elements
        .map(|s| s.split('/'))
        .map(parse_face_index_triple)
        .fold(Ok(vec![]), |mut v, maybe_triple| match maybe_triple {
            Ok(triple) => {
                v.as_mut().map(|v| v.push(triple));
                v
            }
            Err(e) => Err(e),
        })?;
    println!("{:?}", &triples);
    if triples.len() < 3 {
        Err(anyhow!("Face with less than 3 vertices: {:?}", triples))
    } else if triples.len() == 3 {
        triples
    } else {
        // TODO: Face is not a triangle, transform into a set of triangles
        // e.g. [0, 1, 2, 3, 4] => [_0, 1, 2_, _0, 2, 3_, _0, 3, 4_]
    }
}

fn parse_face_index_triple<'a, T: Iterator<Item = &'a str>>(
    mut elements: T,
) -> Result<FaceIndexTriple> {
    let v = elements
        .next()
        .ok_or(anyhow!("v not found"))?
        .parse::<u32>()?;
    let vt = elements
        .next()
        .ok_or(anyhow!("vt not found"))?
        .parse::<u32>()?;
    let vn = elements
        .next()
        .ok_or(anyhow!("vn not found"))?
        .parse::<u32>()?;

    Ok((v, vt, vn))
}
