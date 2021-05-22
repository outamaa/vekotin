use crate::compression::zlib;
use crate::digest::{Crc32, DigestReader};
use anyhow::{anyhow, bail, Result};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str;

//
// Public interface
//
#[derive(PartialEq, Debug)]
pub struct Png {
    pub width: u32,
    pub height: u32,
    pub bit_depth: BitDepth,
    pub color_type: ColorType,
    pub bytes_per_pixel: u32,
    pub data: Vec<u8>,
}

impl Png {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Png> {
        let f = File::open(path)?;
        // TODO for some reason reading chunk type fails if capacity is a bit below this, investigate
        let mut reader = DigestReader::new(BufReader::new(f), Crc32::new());

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

        let image_size: usize = (ihdr.width * ihdr.height * ihdr.bytes_per_pixel) as usize;
        let mut image: Vec<u8> = vec![0; image_size];

        apply_filters(&ihdr, &mut decompressed_data, &mut image)?;

        Ok(Png {
            width: ihdr.width,
            height: ihdr.height,
            bit_depth: ihdr.bit_depth,
            color_type: ihdr.color_type,
            bytes_per_pixel: ihdr.bytes_per_pixel,
            data: image,
        })
    }
}
//
// PNG file header
//

fn read_png_header<R: Read>(reader: &mut R) -> Result<()> {
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

fn read_chunk_type<R: Read>(reader: &mut R) -> Result<ChunkType> {
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

fn read_chunk_length_and_type<R: Read>(
    reader: &mut DigestReader<R, Crc32>,
) -> Result<(u32, ChunkType)> {
    let length = read_u32(reader)?;
    reader.reset_digest();
    Ok((length, read_chunk_type(reader)?))
}

//
// IHDR
//

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum BitDepth {
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

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ColorType {
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
    bytes_per_pixel: u32,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    interlace_method: InterlaceMethod,
}

fn read_ihdr<R: Read>(reader: &mut DigestReader<R, Crc32>) -> Result<IHDR> {
    let (chunk_length, chunk_type) = read_chunk_length_and_type(reader)?;

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
    let bytes_per_pixel = bytes_per_pixel(&color_type, &bit_depth)?;

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
    if interlace_method != InterlaceMethod::None {
        bail!("Can't handle interlacing yet");
    }

    check_crc(reader)?;

    let ihdr = IHDR {
        width,
        height,
        bit_depth,
        color_type,
        bytes_per_pixel,
        compression_method,
        filter_method,
        interlace_method,
    };

    Ok(ihdr)
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
    use FilterAlgorithm::*;
    let bpp = ihdr.bytes_per_pixel;
    let scanline_len = ihdr.width as usize * bpp as usize;

    for (scanline_idx, filter_and_scanline) in
        decompressed_data.chunks(scanline_len + 1).enumerate()
    {
        let filter_algorithm = FilterAlgorithm::try_from(filter_and_scanline[0])?;
        let scanline = &filter_and_scanline[1..];

        match filter_algorithm {
            Sub => {
                for (byte_idx, byte) in scanline.iter().enumerate() {
                    let left = raw(
                        image,
                        scanline_len,
                        scanline_idx,
                        byte_idx as i32 - bpp as i32,
                    );

                    let image_idx = scanline_len * scanline_idx + byte_idx;
                    image[image_idx] = byte.wrapping_add(left);
                }
            }
            Up => {
                for (byte_idx, byte) in scanline.iter().enumerate() {
                    let prior_byte = prior(image, scanline_len, scanline_idx, byte_idx as i32);

                    let image_idx = scanline_len * scanline_idx + byte_idx;
                    image[image_idx] = byte.wrapping_add(prior_byte);
                }
            }
            Average => {
                for (byte_idx, byte) in scanline.iter().enumerate() {
                    let raw_byte: u32 = raw(
                        image,
                        scanline_len,
                        scanline_idx,
                        byte_idx as i32 - bpp as i32,
                    ) as u32;
                    let prior_byte: u32 =
                        prior(image, scanline_len, scanline_idx, byte_idx as i32) as u32;
                    let avg_byte: u8 = ((raw_byte + prior_byte) / 2) as u8;

                    let image_idx = scanline_len * scanline_idx + byte_idx;
                    image[image_idx] = byte.wrapping_add(avg_byte);
                }
            }
            Paeth => {
                for (byte_idx, byte) in scanline.iter().enumerate() {
                    let left = raw(
                        image,
                        scanline_len,
                        scanline_idx,
                        byte_idx as i32 - bpp as i32,
                    );
                    let above = prior(image, scanline_len, scanline_idx, byte_idx as i32);
                    let above_left = prior(
                        image,
                        scanline_len,
                        scanline_idx,
                        byte_idx as i32 - bpp as i32,
                    );
                    let paeth = paeth_predictor(left, above, above_left);

                    let image_idx = scanline_len * scanline_idx + byte_idx;
                    image[image_idx] = byte.wrapping_add(paeth);
                }
            }
            _ => {
                let image_idx = scanline_len * scanline_idx;
                image[image_idx..image_idx + scanline_len]
                    .as_mut()
                    .write(&scanline[1..])?;
            }
        }
    }

    Ok(())
}

// raw, unfiltered byte from the prior scanline
fn prior(image: &mut Vec<u8>, scanline_len: usize, scanline_idx: usize, byte_idx: i32) -> u8 {
    if scanline_idx == 0 || byte_idx < 0 {
        0
    } else {
        image[scanline_len * (scanline_idx - 1) + byte_idx as usize]
    }
}

fn raw(image: &mut Vec<u8>, scanline_len: usize, scanline_idx: usize, byte_idx: i32) -> u8 {
    assert!(
        byte_idx < scanline_len as i32,
        "{}, {}",
        byte_idx,
        scanline_len
    );
    if byte_idx < 0 {
        0
    } else {
        image[scanline_len * scanline_idx + byte_idx as usize]
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;
    let pa = (p - a as i32).abs();
    let pb = (p - b as i32).abs();
    let pc = (p - c as i32).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn bytes_per_pixel(color_type: &ColorType, bit_depth: &BitDepth) -> Result<u32> {
    match (color_type, bit_depth) {
        (ColorType::Grayscale, BitDepth::Bits1) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits2) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits4) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits8) => Ok(1),
        (ColorType::Grayscale, BitDepth::Bits16) => Ok(2),
        (ColorType::RGB, BitDepth::Bits8) => Ok(3),
        (ColorType::RGB, BitDepth::Bits16) => Ok(6),
        (ColorType::Palette, _) => bail!("Can't handle palettes yet"),
        (ColorType::GrayscaleAlpha, BitDepth::Bits8) => Ok(2),
        (ColorType::GrayscaleAlpha, BitDepth::Bits16) => Ok(4),
        (ColorType::RGBA, BitDepth::Bits8) => Ok(4),
        (ColorType::RGBA, BitDepth::Bits16) => Ok(8),
        _ => bail!(
            "Unknown combination of color type and bit_depth: {:?}, {:?}",
            color_type,
            bit_depth
        ),
    }
}

fn process_chunk<R: Read>(
    mut reader: &mut DigestReader<BufReader<R>, Crc32>,
    mut compressed_data: &mut Vec<u8>,
) -> Result<bool> {
    let (chunk_length, chunk_type) = read_chunk_length_and_type(&mut reader)?;
    match chunk_type {
        ChunkType::IEND => return Ok(false),
        ChunkType::IDAT => {
            // Read data chunk to `data`
            let chunk_reader = reader.by_ref();
            chunk_reader
                .take(chunk_length.into())
                .read_to_end(&mut compressed_data)?;
        }
        ChunkType::PLTE => bail!("Can't handle PNGs with palette yet!"),
        ChunkType::IHDR => bail!("Encountered a second IHDR chunk"),
        _ => {
            skip_bytes(&mut reader.get_mut(), chunk_length)?;
        }
    }
    check_crc(&mut reader)?;
    Ok(true)
}

//
// Helpers
//

fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut b = [0; 4];
    reader.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}

fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut b = [0; 1];
    reader.read_exact(&mut b)?;
    Ok(b[0])
}

fn check_crc<R: Read>(reader: &mut DigestReader<R, Crc32>) -> Result<()> {
    let crc_from_reader = reader.digest();
    let crc = read_u32(reader)?;
    if crc != crc_from_reader {
        // bail!("Invalid CRC, {} != {}", crc, crc_from_reader);
    }
    Ok(())
}

// For development
fn skip_bytes<R: Read>(reader: &mut R, n: u32) -> Result<()> {
    let mut v = vec![0 as u8; n as usize];
    reader.read_exact(&mut v)?;
    Ok(())
}
