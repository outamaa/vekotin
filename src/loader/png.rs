use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::str;
use anyhow::{Result, bail};
use crate::compression::zlib;
//
// Helpers
//

fn read_u32<R: Read>(reader: &mut BufReader<R>) -> Result<u32> {
    let mut b = [0; 4];
    reader.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

fn read_u8<R: Read>(reader: &mut BufReader<R>) -> Result<u8> {
    let mut b = [0; 1];
    reader.read_exact(&mut b)?;
    Ok(b[0])
}

// TODO actually check CRC
fn skip_crc<R: Read>(reader: &mut BufReader<R>) -> Result<()> {
    read_u32(reader)?;
    Ok(())
}

// For development
fn skip_bytes<R: Read>(reader: &mut BufReader<R>, n: u32) {
    reader.consume(n as usize);
}

//
// PNG file header
//

fn read_png_header<R: Read>(reader: &mut BufReader<R>) -> Result<()> {
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

//
// Chunks
//

#[derive(PartialEq, Debug)]
enum ChunkType {
    IHDR,
    PLTE,
    IDAT,
    IEND,
    Ancillary(String),
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

fn read_chunk_length_and_type<R: Read>(reader: &mut BufReader<R>) -> Result<(u32, ChunkType)> {
    Ok( (read_u32(reader)?, read_chunck_type(reader)?))
}


//
// IHDR
//

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
enum CompressionMethod {
    Deflate,
    Unknown,
}

impl From<u8> for CompressionMethod {
    fn from(b: u8) -> Self {
        use CompressionMethod::*;

        match b {
            0 => Deflate,
            _ => Unknown,
        }
    }
}

#[derive(PartialEq, Debug)]
enum FilterMethod {
    Adaptive,
    Unknown,
}

impl From<u8> for FilterMethod {
    fn from(b: u8) -> Self {
        use FilterMethod::*;

        match b {
            0 => Adaptive,
            _ => Unknown,
        }
    }
}

#[derive(PartialEq, Debug)]
enum InterlaceMethod {
    None,
    Adam7,
    Unknown,
}

impl From<u8> for InterlaceMethod {
    fn from(b: u8) -> Self {
        use InterlaceMethod::*;

        match b {
            0 => None,
            1 => Adam7,
            _ => Unknown,
        }
    }
}

#[derive(PartialEq, Debug)]
struct IHDR {
    width: u32,
    height: u32,
    bit_depth: BitDepth,
    color_type: ColorType,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    interlace_method: InterlaceMethod,
}

fn read_ihdr<R: Read>(reader: &mut BufReader<R>) -> Result<IHDR> {
    let chunk_length = read_u32(reader)?;
    let chunk_type = read_chunck_type(reader)?;

    if chunk_type != ChunkType::IHDR {
        bail!("First chunk must be IHDR, was {:?}", chunk_type);
    }

    if chunk_length != 13 {
        bail!("IHDR chunk length must be 13, not {}", chunk_length);
    }

    let width = read_u32(reader)?;
    let height = read_u32(reader)?;
    let bit_depth = BitDepth::from(read_u8(reader)?);
    let color_type = ColorType::from(read_u8(reader)?);

    let compression_method_byte = read_u8(reader)?;
    let compression_method = CompressionMethod::from(compression_method_byte);
    if compression_method == CompressionMethod::Unknown {
        bail!("Unknown compression method {}", compression_method_byte);
    }

    let filter_method_byte = read_u8(reader)?;
    let filter_method = FilterMethod::from(filter_method_byte);
    if filter_method == FilterMethod::Unknown {
        bail!("Unknown filter method {}", filter_method_byte);
    }


    let interlace_method_byte = read_u8(reader)?;
    let interlace_method = InterlaceMethod::from(interlace_method_byte);
    if interlace_method == InterlaceMethod::Unknown {
        bail!("Unknown interlace method {}", interlace_method_byte);
    }

    skip_crc(reader)?;

    let ihdr = IHDR {
        width,
        height,
        bit_depth,
        color_type,
        compression_method,
        filter_method,
        interlace_method,
    };

    Ok(ihdr)
}


//
// Public interface
//

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    // PNG header
    read_png_header(&mut reader)?;

    // IHDR must be the first chunk.
    let ihdr = read_ihdr(&mut reader)?;
    println!("{:?}", ihdr);


    let mut data: Vec<u8> = Vec::new();

    // Loop through the chunks
    loop {
        let (chunk_length, chunk_type) = read_chunk_length_and_type(&mut reader)?;
        match chunk_type {
            ChunkType::IEND => break,
            ChunkType::IDAT => {
                // Read data chunk to `data`
                let chunk_reader = reader.by_ref();
                chunk_reader.take(chunk_length.into()).read_to_end(&mut data)?;
            },
            ChunkType::PLTE => bail!("Can't handle PNGs with palette yet!"),
            ChunkType::IHDR => bail!("Encountered a second IHDR chunk"),
            _ => {
                println!("{:?}", chunk_type);
                skip_bytes(&mut reader, chunk_length);
            },
        }
        skip_crc(&mut reader)?;
    }

    println!("Data length: {}", data.len());
    zlib::decompress(&data)?;

    Ok(())
}