mod huffman;

pub use huffman::HuffmanAlphabet;

use anyhow::{bail, Result};
use fiddling::*;
use std::io::{Read, Write};

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

impl From<u8> for BlockHeader {
    fn from(b: u8) -> Self {
        use CompressionType::*;
        let is_final = b & 1 == 1;
        let btype = first_n_bits(b >> 1, 2);
        let compression_type = match btype {
            0b00 => NoCompression,
            0b01 => FixedHuffman,
            0b10 => DynamicHuffman,
            0b11 => Reserved,
            _ => unreachable!(),
        };
        Self {
            is_final,
            compression_type,
        }
    }
}

// Return the three block header bits as
fn read_block_header<R: Read>(bits: &mut BitStream<R>) -> Result<BlockHeader> {
    let header_bits = bits.read_bits(3, BitOrder::LsbFirst)?;
    Ok(BlockHeader::from(header_bits as u8))
}

fn copy_bytes<R: Read, W: Write>(r: &mut R, w: &mut W) -> Result<()> {
    let mut buf = [0u8; 1024];
    let mut bytes_written = r.read(&mut buf[..])?;
    while bytes_written != 0 {
        w.write_all(&buf[..bytes_written])?;
        bytes_written = r.read(&mut buf[..])?;
    }
    Ok(())
}

fn copy_uncompressed_block<R: Read, W: Write>(
    bits: &mut BitStream<R>,
    out_bytes: &mut W,
) -> Result<()> {
    bits.skip_to_start_of_byte();

    let len = bits.read_u16_le()?;
    let nlen = bits.read_u16_le()?;

    if len & nlen != 0 {
        bail!("LEN & NLEN != 0");
    }

    let mut bytes_to_read = bits.get_mut().take(len as u64);
    copy_bytes(&mut bytes_to_read, out_bytes)?;
    Ok(())
}

pub fn decompress_blocks(in_bytes: &[u8], out_buf: &mut Vec<u8>) -> Result<()> {
    use CompressionType::*;
    let mut bits = BitStream::new(in_bytes);
    'block: loop {
        let block_header = read_block_header(&mut bits)?;

        println!("{:?}", block_header.compression_type);
        match block_header.compression_type {
            NoCompression => {
                copy_uncompressed_block(&mut bits, out_buf)?;
            }
            FixedHuffman => {
                huffman::copy_static_huffman_block(&mut bits, out_buf)?;
            }
            DynamicHuffman => {
                huffman::copy_dynamic_huffman_block(&mut bits, out_buf)?;
            }
            Reserved => bail!("Invalid compression type, Reserved"),
        }

        if block_header.is_final {
            println!("Final block! We're done!");
            break 'block;
        }
    }

    Ok(())
}
