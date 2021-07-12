use crate::fiddling::BitOrder::{LSBFirst, MSBFirst};
use crate::fiddling::Fiddler;
use anyhow::{bail, Result};
use std::io::{Read, Write};

pub fn copy_dynamic_huffman_block<R: Read, W: Write>(
    bits: &mut Fiddler<R>,
    out_bytes: &mut W,
) -> Result<()> {
    bits.skip_to_next_byte();

    let hlit = bits.read_bits(5, LSBFirst)? + 257;
    let hdist = bits.read_bits(5, LSBFirst)? + 1;
    let hclen = bits.read_bits(4, LSBFirst)? + 4;

    let mut code_lengths = vec![0u8; hclen as usize];
    for i in 0..hclen as usize {
        code_lengths[i] = bits.read_bits(3, LSBFirst)? as u8;
    }
    bail!(
        "hlit = {}, hdist = {}, hclen = {}, code_lengths = {:?}",
        hlit,
        hdist,
        hclen,
        code_lengths
    );
}
