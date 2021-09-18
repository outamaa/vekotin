use anyhow::{anyhow, Result};
use math::{Vec2f, Vec3f};
use std::cmp::Ordering;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

//
// TODO: For now, no support for e.g. 4D coordinates for vertices or 3D textures
//

//
// Public interface
//

type TriangleIndexTriple = (u32, u32, u32);

#[derive(PartialEq, Debug)]
pub struct Obj {
    // TODO: use points for vertices and uvs after implementing Transform
    pub vertices: Vec<Vec3f>,
    pub uvs: Vec<Vec2f>,
    pub normals: Vec<Vec3f>,
    pub vertex_index_triples: Vec<TriangleIndexTriple>,
    pub uv_index_triples: Vec<TriangleIndexTriple>,
    pub normal_index_triples: Vec<TriangleIndexTriple>,
}

impl Obj {
    pub fn new() -> Self {
        Obj {
            vertices: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            vertex_index_triples: Vec::new(),
            uv_index_triples: Vec::new(),
            normal_index_triples: Vec::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = File::open(path)?;
        let mut obj = Obj::new();

        for (line_num, maybe_line) in io::BufReader::new(f).lines().enumerate() {
            let line = maybe_line?;
            if line.is_empty() {
                continue;
            }
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
                    obj.normals.push(parse_vec3f(elems)?);
                }
                "f" => {
                    for triple in parse_face(elems)?.windows(3) {
                        obj.vertex_index_triples.push((
                            triple[0].0 - 1,
                            triple[1].0 - 1,
                            triple[2].0 - 1,
                        ));
                        obj.uv_index_triples.push((
                            triple[0].1 - 1,
                            triple[1].1 - 1,
                            triple[2].1 - 1,
                        ));
                        obj.normal_index_triples.push((
                            triple[0].2 - 1,
                            triple[1].2 - 1,
                            triple[2].2 - 1,
                        ));
                    }
                }
                "#" => {}
                _ => {
                    println!("Skipping line {}: {}", line_num, line);
                }
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
    elements: T, // ["1/2/3", "2/3/4", ...]
) -> Result<Vec<FaceIndexTriple>> {
    let triples = elements
        .map(|s| s.split('/'))
        .map(parse_face_index_triple)
        .try_fold(vec![], |mut v, maybe_triple| match maybe_triple {
            Ok(triple) => {
                v.push(triple);
                Ok(v)
            }
            Err(e) => Err(e),
        })?;

    match triples.len().cmp(&3) {
        Ordering::Less => Err(anyhow!("Face with less than 3 vertices: {:?}", triples)),
        Ordering::Equal => Ok(triples),
        Ordering::Greater => {
            // Face is not a triangle, transform into a set of triangles
            // e.g. [0, 1, 2, 3, 4] => [_0, 1, 2_, _0, 2, 3_, _0, 3, 4_]
            let start_triple = triples[0];
            let triangled_triples: Vec<FaceIndexTriple> = triples[1..]
                .windows(2)
                // TODO: How to do this without vec! and heap allocation
                // why not just plain old for loop?
                .flat_map(|w| vec![start_triple, w[0], w[1]])
                .collect();
            Ok(triangled_triples)
        }
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
