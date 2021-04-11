use crate::math::{Vec2f, Vec3f};
use anyhow::{anyhow, bail, Result};
use regex::Regex;
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

        lazy_static! {
            static ref r_v: Regex = Regex::new(r"^v (\d+.\d+) (\d+.\d+) (\d+.\d+)").unwrap();
        }
        for (line_num, maybe_line) in io::BufReader::new(f).lines().enumerate() {
            let line = maybe_line?;
            r_v.is_match(&line);
        }
        Ok(obj)
    }
}
