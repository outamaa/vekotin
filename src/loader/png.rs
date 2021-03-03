use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::str;
use anyhow::{Result, bail};

fn read_chunk_length<R: Read>(reader: &mut BufReader<R>) -> Result<u32> {
    let mut b = [0; 4];
    reader.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

#[derive(PartialEq, Debug)]
enum ChunkType {
    IHDR,
    PLTE,
    IDAT,
    IEND,
    Ancillary(String),
}

struct Chunk<R: Read> {
    chunk_type: ChunkType,
    reader: BufReader<R>, 
}


fn read_chunck_type<R: Read>(reader: &mut BufReader<R>) -> Result<ChunkType> {
    use ChunkType::*;
    let mut b = [0; 4];
    reader.read_exact(&mut b)?;

    let chunk_type_str = str::from_utf8(&b)?;

    let chunk_type = match chunk_type_str {
        "IHDR" => IHDR,
        "PLTE" => PLTE,
        "IDAT" => IDAT,
        "IEND" => IEND,
        _ => Ancillary(chunk_type_str.to_string()),
    };


    Ok(chunk_type)
}

fn read_header<R: Read>(reader: &mut BufReader<R>) -> Result<()> {
    let mut b = [0; 8];
    reader.read_exact(&mut b)?;

    if b[0] != 0x89
    || b[1] != 0x50
    || b[2] != 0x4E
    || b[3] != 0x47
    || b[4] != 0x0D
    || b[5] != 0x0A
    || b[6] != 0x1A
    || b[7] != 0x0A
    {
        bail!("Not a PNG header: {:?}", b);
    }

    Ok(())
}

#[derive(PartialEq, Debug)]
enum BitDepth {
    Bits1,
    Bits2,
    Bits4,
    Bits8,
    Bits16,
    Invalid,
}

impl From<u8> for BitDepth {
    fn from(b: u8) -> Self {
        use BitDepth::*;

        match b {
            1  => Bits1,
            2  => Bits2,
            4  => Bits4,
            8  => Bits8,
            16 => Bits16,
            _  => Invalid,
        }
    }
}

#[derive(PartialEq, Debug)]
enum ColorType {
    Grayscale,
    RGB,
    Palette,
    GrayscaleAlpha,
    RGBA,
    Invalid,
}

impl From<u8> for ColorType {
    fn from(b: u8) -> Self {
        use ColorType::*;

        match b {
            0 => Grayscale,
            2 => RGB,
            3 => Palette,
            4 => GrayscaleAlpha,
            6 => RGBA,
            _ => Invalid,
        }
    }
}

#[derive(PartialEq, Debug)]
struct IHDR {
    width: u32,
    height: u32,
    bit_depth: BitDepth,
    color_type: ColorType,
}


fn read_ihdr<R: Read>(reader: &mut BufReader<R>) -> Result<IHDR> {

    let chunk_length = read_chunk_length(reader)?;
    let chunk_type = read_chunck_type(reader)?;

    if chunk_type != ChunkType::IHDR {
        bail!("First chunk must be IHDR, was {:?}", chunk_type);
    }

    let ihdr = IHDR {
        width: 0,
        height: 0,
        bit_depth: BitDepth::Invalid,
        color_type: ColorType::Invalid,
    };
    
    Ok(ihdr)
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    read_header(&mut reader)?;
    // IHDR must be the first chunk.
    let ihdr = read_ihdr(&mut reader)?;

    println!("{:?}", ihdr);

    Ok(())
}