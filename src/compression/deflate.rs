use anyhow::{Result, bail};
use crate::fiddling::*;
use std::io::Write;

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
            _ => unreachable!()
        };
        Self {
            is_final,
            compression_type,
        }
    }    
}

// Return the three block header bits as 
fn read_block_header(bit_idx: usize, bytes: &[u8]) -> (usize, BlockHeader) {
    let header_bits = n_bits_by_index(bytes, 3, bit_idx, BitOrder::LSBFirst);
    (3, BlockHeader::from(header_bits as u8))
}

fn copy_uncompressed_block<W: Write>(bit_idx: usize, in_bytes: &[u8], out_bytes: &mut W) -> Result<usize> {
    let mut read_bits = 8 - bit_idx % 8;
    let start_byte = (bit_idx + read_bits) / 8;
    read_bits += 32; // LEN and NLEN
    
    let buf = [in_bytes[start_byte], in_bytes[start_byte + 1]];
    let len = u16::from_le_bytes(buf);
    
    let buf = [in_bytes[start_byte + 2], in_bytes[start_byte + 3]];
    let nlen = u16::from_le_bytes(buf);
    
    if len & nlen != 0 {
        bail!("LEN & NLEN != 0");
    }

    let bytes_written = out_bytes.write(&in_bytes[start_byte + 4..start_byte + (len + 4) as usize])?;

    Ok(read_bits + 8 * bytes_written)
}

pub fn decompress_blocks<W: Write>(in_bytes: &[u8], out_bytes: &mut W) -> Result<()> {
    use CompressionType::*;

    let mut bit_idx = 0;
    'block: loop {
        let (read_bits, block_header) = read_block_header(bit_idx, in_bytes);
        bit_idx = bit_idx + read_bits;

        match block_header.compression_type {
            NoCompression => {
                let read_bits = copy_uncompressed_block(bit_idx, in_bytes, out_bytes)?;
                bit_idx += read_bits;
            },
            Reserved => bail!("Invalid compression type, Reserved"),
            _ => bail!("Can't really decompress yet, sorry!")
        }

        if block_header.is_final {
            println!("Final block! We're done!");
            break 'block;
        }
    }

    Ok(())
}