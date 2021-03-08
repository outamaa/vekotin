use std::io::prelude::*;
use std::io::BufReader;
use std::convert::TryFrom;
use std::fs::File;
use std::path::Path;
use std::str;
use anyhow::{Result, bail, anyhow};
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
    Ok((read_u32(reader)?, read_chunck_type(reader)?))
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
            1 => Bits1,
            2 => Bits2,
            4 => Bits4,
            8 => Bits8,
            16 => Bits16,
            _ => Invalid,
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
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Image> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    // PNG header
    read_png_header(&mut reader)?;

    // IHDR must be the first chunk.
    let ihdr = read_ihdr(&mut reader)?;
    println!("{:?}", ihdr);


    // Loop through the chunks, copying data to `compressed_data`
    let mut compressed_data: Vec<u8> = Vec::new();
    while process_chunk(&mut reader, &mut compressed_data)? {}

    let mut decompressed_data: Vec<u8> = Vec::new();
    zlib::decompress(&compressed_data, &mut decompressed_data)?;

    let image_size: usize = (ihdr.width * ihdr.height * 4) as usize; // Assumes RGBA for now
    let mut image: Vec<u8> = vec![0; image_size];

    apply_filters(&ihdr, &mut decompressed_data, &mut image);

    Ok(Image {
        width: ihdr.width,
        height: ihdr.height,
        data: image,
    })
}


#[derive(Debug, PartialEq)]
enum FilterAlgorithm {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

impl TryFrom<u8> for FilterAlgorithm {
    type Error = anyhow::Error;

    fn try_from(b: u8) -> Result<Self, Self::Error> {
        use FilterAlgorithm::*;
        match b {
            0 => Ok(None),
            1 => Ok(Sub),
            2 => Ok(Up),
            3 => Ok(Average),
            4 => Ok(Paeth),
            _ => Err(anyhow!("Unknown filter algorithm")),
        }
    }
}

fn apply_filters(ihdr: &IHDR, decompressed_data: &mut Vec<u8>, image: &mut Vec<u8>) -> Result<()> {
    // Copy unfiltered data to `image`
    // TODO no need to copy, but first implement filters
    let mut scanline_start_idx = 0;
    let bpp = bytes_per_pixel(ihdr)? as usize;
    let scanline_len = ihdr.width as usize * bpp;

    for (scanline_idx, scanline) in decompressed_data.chunks(scanline_len + 1).enumerate() {
        let filter_algorithm = FilterAlgorithm::try_from(scanline[0])?;
        println!("Filter algorithm: {:?}", filter_algorithm);
        match filter_algorithm {
            Sub => {
                for (byte_idx, byte) in scanline[1..].iter().enumerate() {
                    let image_idx = scanline_len * scanline_idx + byte_idx;
                    let sub_image_byte = if image_idx < bpp { 0 } else { image[image_idx - bpp] };
                    image[image_idx] = byte.wrapping_add(sub_image_byte);
                }
            },
            Up => {
                for (byte_idx, byte) in scanline[1..].iter().enumerate() {
                    let image_idx = scanline_len * scanline_idx + byte_idx;
                    let prior_idx = image_idx - scanline_len;
                    let prior_byte = if image_idx < scanline_len { 0 } else { image[prior_idx] };
                    image[image_idx] = byte.wrapping_add(prior_byte);
                }
            }
            _ => {
                let image_idx = scanline_len * scanline_idx;
                image[image_idx..image_idx + scanline_len].as_mut().write(&scanline[1..]);
            }
        }
    }

    Ok(())
}

fn bytes_per_pixel(ihdr: &IHDR) -> Result<u32> {
    match (&ihdr.color_type, &ihdr.bit_depth) {
        (ColorType::Grayscale, BitDepth::Bits1) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits2) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits4) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits8) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits16) => Ok(2),
        (ColorType::RGB, BitDepth::Bits8) => Ok(1),
        (ColorType::RGB, BitDepth::Bits8) => Ok(2),
        (ColorType::Palette, _) => bail!("Can't handle palette"),
        (ColorType::GrayscaleAlpha, BitDepth::Bits8) => Ok(2),
        (ColorType::GrayscaleAlpha, BitDepth::Bits16) => Ok(4),
        (ColorType::RGBA, BitDepth::Bits8) => Ok(4),
        (ColorType::RGBA, BitDepth::Bits16) => Ok(8),
        (_, _) => bail!("Unknown combination of color type and bit_depth")
    }
}

fn process_chunk(mut reader: &mut BufReader<File>, mut compressed_data: &mut Vec<u8>) -> Result<bool> {
    let (chunk_length, chunk_type) = read_chunk_length_and_type(&mut reader)?;
    match chunk_type {
        ChunkType::IEND => return Ok(false),
        ChunkType::IDAT => {
            // Read data chunk to `data`
            let chunk_reader = reader.by_ref();
            chunk_reader.take(chunk_length.into()).read_to_end(&mut compressed_data)?;
        }
        ChunkType::PLTE => bail!("Can't handle PNGs with palette yet!"),
        ChunkType::IHDR => bail!("Encountered a second IHDR chunk"),
        _ => {
            println!("{:?}", chunk_type);
            skip_bytes(&mut reader, chunk_length);
        }
    }
    skip_crc(&mut reader)?;
    Ok(true)
}
