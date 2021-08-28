use crate::compression::deflate;
use anyhow::{bail, Result};

#[derive(PartialEq, Debug)]
enum CompressionMethod {
    Deflate { window_size: u16 },
    Unknown,
}

impl From<u8> for CompressionMethod {
    fn from(b: u8) -> Self {
        use CompressionMethod::*;
        let cm = b & 0b00001111; // First 4 bits
        let cinfo = b >> 4; // Last 4 bits

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
        let preset_dictionary = (b & 0b0001_0000) == 0b0001_0000;
        let flevel = b >> 6;
        let compression_level = match flevel {
            0 => Level1,
            1 => Level2,
            2 => Level3,
            3 => Level4,
            _ => unreachable!(),
        };
        Flags {
            preset_dictionary,
            compression_level,
        }
    }
}

fn check_cmf_flg(cmf: u8, flg: u8) -> bool {
    (256 * cmf as u32 + flg as u32) % 31 == 0
}

pub fn decompress(in_bytes: &[u8], out_buf: &mut Vec<u8>) -> Result<()> {
    let compression_method = CompressionMethod::from(in_bytes[0]);
    println!("{:?}", compression_method);
    let flags = Flags::from(in_bytes[1]);
    println!("{:?}", flags);
    if !check_cmf_flg(in_bytes[0], in_bytes[1]) {
        bail!("FCHECK failed");
    }

    deflate::decompress_blocks(&in_bytes[2..], out_buf)?;

    Ok(())
}
