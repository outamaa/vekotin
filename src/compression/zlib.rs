use anyhow::{Result, bail};
use std::cmp;

#[derive(PartialEq, Debug)]
enum CompressionMethod {
    Deflate {
        window_size: u16,
    },
    Unknown,
}

impl From<u8> for CompressionMethod {
    fn from(b: u8) -> Self {
        use CompressionMethod::*;
        let cm = b & 0b00001111;
        let cinfo = b >> 4;

        if cm == 8 {
            // TODO: see http://optipng.sourceforge.net/pngtech/zlib-spec-correction.html
            let window_size = u16::pow(2u16, cinfo as u32 + 8);
            Deflate { window_size }
        } else {
            Unknown
        }
    }
}

#[derive(PartialEq, Debug)]
enum CompressionLevel {
    Level1,
    Level2,
    Level3,
    Level4,
}

#[derive(PartialEq, Debug)]
struct Flags {
    preset_dictionary: bool,
    compression_level: CompressionLevel,
}

impl From<u8> for Flags {
    fn from(b: u8) -> Self {
        use CompressionLevel::*;
        let preset_dictionary = (b & 0x08) == 5;
        let flevel = b >> 6;
        let compression_level = match flevel {
            0 => Level1,
            1 => Level2,
            2 => Level3,
            3 => Level4,
            _ => unreachable!()
        };
        Flags {
            preset_dictionary,
            compression_level,
        }
    }
}

#[derive(PartialEq, Debug)]
enum CompressionType {
    NoCompression,
    FixedHuffman,
    DynamicHuffman,
    Reserved,
}

#[derive(PartialEq, Debug)]
struct BlockHeader {
    is_final: bool,
    compression_type: CompressionType,
}

fn check_cmf_flg(cmf: u8, flg: u8) -> bool {
    (256 * cmf as u32 + flg as u32) % 31 == 0
}

static FIRST_N_BITS: &'static [u8] = &[
    0b00000000,
    0b00000001,
    0b00000011,
    0b00000111,
    0b00001111,
    0b00011111,
    0b00111111,
    0b01111111,
    0b11111111,
];

/// # Examples
/// 
/// ```rust
/// use vekotin::compression::zlib;
/// 
/// let bits = zlib::first_n_bits(0b11111111, 0);
/// assert_eq!(bits, 0);
/// 
/// let bits = zlib::first_n_bits(0b11111111, 3);
/// assert_eq!(bits, 0b00000111);
/// 
/// assert_eq!(zlib::first_n_bits(0b11111111, 100),
///            zlib::first_n_bits(0b11111111, 8));
/// ```
pub fn first_n_bits(byte: u8, n: u8) -> u8 {
    byte & FIRST_N_BITS[cmp::min(n, 8) as usize]
}

/// # Examples
/// 
/// ```rust
/// use vekotin::compression::zlib;
/// 
/// let bits = zlib::last_n_bits(0b11111111, 0);
/// assert_eq!(bits, 0);
/// 
/// let bits = zlib::last_n_bits(0b10111111, 3);
/// assert_eq!(bits, 0b00000101);
/// 
/// assert_eq!(zlib::last_n_bits(0b11111111, 100),
///            zlib::last_n_bits(0b11111111, 8));
/// ```
pub fn last_n_bits(byte: u8, n: u8) -> u8 {
    if n >= 8 {
        byte
    } else if n == 0 {
        0
    } else {
        byte >> 8-n
    }
}

/// # Examples
/// 
/// ```rust
/// use vekotin::compression::zlib;
/// ```
pub fn read_n_bits(bit_idx: usize, bytes: &[u8], n_bits: u8) -> u64 {
    let mut byte_idx = bit_idx / 8;
    let within_bit_idx: u8 = (bit_idx % 8) as u8;
    let mut n = n_bits;
    let mut read_bits: u64 = 0;

    if within_bit_idx != 0 {
        read_bits = last_n_bits(bytes[byte_idx], within_bit_idx).into();
        n = n - 8u8 + within_bit_idx;
        byte_idx = byte_idx + 1;
    }

    while n > 0 {
        read_bits = read_bits * 256 + bytes[byte_idx] as u64;
        n = n - 8;
        byte_idx = byte_idx + 1;
    }
    read_bits
}

// Return the three block header bits as 
fn read_block_header(bit_idx: usize, bytes: &[u8]) -> BlockHeader {
    BlockHeader {
        is_final: true,
        compression_type: CompressionType::NoCompression,
    }
}

pub fn decompress(bytes: &[u8]) -> Result<()> {
    let compression_method = CompressionMethod::from(bytes[0]);
    println!("{:?}", compression_method);
    let flags = Flags::from(bytes[1]);
    println!("{:?}", flags);
    if !check_cmf_flg(bytes[0], bytes[1]) {
        bail!("FCHECK failed");
    }

    let mut bit_idx: usize = 16; // Beginning of byte 2
    loop {
        let block_header = read_block_header(bit_idx, bytes);
        bit_idx = bit_idx + 3;

        println!("{:?}", block_header);
        if block_header.is_final {
            break;
        }
    }

    Ok(())
}