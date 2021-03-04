use anyhow::{Result, bail};

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

fn check_cmf_flg(cmf: u8, flg: u8) -> bool {
    (256 * cmf as u32 + flg as u32) % 31 == 0
}

pub fn decompress(bytes: &[u8]) -> Result<()> {
    let compression_method = CompressionMethod::from(bytes[0]);
    println!("{:?}", compression_method);
    let flags = Flags::from(bytes[1]);
    println!("{:?}", flags);
    if !check_cmf_flg(bytes[0], bytes[1]) {
        bail!("FCHECK failed");
    }

    Ok(())
}