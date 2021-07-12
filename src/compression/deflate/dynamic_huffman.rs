use crate::fiddling::BitOrder::{LSBFirst, MSBFirst};
use crate::fiddling::Fiddler;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::{Read, Write};

const CODE_LENGTH_ALPHABET_INDICES: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Clone, Debug, PartialEq)]
pub struct HuffmanAlphabet<S: Copy + Ord> {
    lookup_table: Vec<(S, u8, u16)>,
}

impl<S: Copy + Ord> HuffmanAlphabet<S> {
    /// # Examples
    ///
    /// ```rust
    /// use vekotin::compression::deflate::HuffmanAlphabet;
    ///
    /// let code_lengths = [('A', 3u8), ('B', 3), ('C', 3), ('D', 3), ('E', 3), ('F', 2), ('G', 4), ('H', 4)];
    ///
    /// let alphabet = HuffmanAlphabet::from_code_lengths(&code_lengths[..]);
    /// assert_eq!(alphabet, HuffmanAlphabet::from_code_lengths(&code_lengths[1..]));
    /// ```
    pub fn from_code_lengths(code_lengths: &[(S, u8)]) -> Self {
        let max_code_length = *code_lengths.iter().map(|(_, len)| len).max().unwrap();
        let mut bl_count = vec![0; max_code_length as usize + 1];
        code_lengths.iter().for_each(|&(_, x)| {
            bl_count[x as usize] += 1;
        });
        assert_eq!(bl_count[0], 0);

        let mut next_code = vec![0u16; bl_count.len() + 1];
        let mut code = 0;
        for bits in 1..bl_count.len() + 1 {
            code = (code + bl_count[(bits - 1) as usize]) << 1;
            next_code[bits] = code;
        }

        let mut tree: Vec<(S, u8, u16)> =
            code_lengths.iter().map(|&(s, len)| (s, len, 0)).collect();

        for n in 0..tree.len() {
            let len = tree[n].1;
            if len != 0 {
                tree[n].2 = next_code[len as usize];
                next_code[len as usize] += 1;
            }
        }
        HuffmanAlphabet { lookup_table: tree }
    }
}

pub fn copy_dynamic_huffman_block<R: Read, W: Write>(
    bits: &mut Fiddler<R>,
    out_bytes: &mut W,
) -> Result<()> {
    let hlit = bits.read_bits(5, LSBFirst)? + 257;
    let hdist = bits.read_bits(5, LSBFirst)? + 1;
    let hclen = bits.read_bits(4, LSBFirst)? + 4;

    let mut code_lengths = vec![0u8; 19];
    for i in 0..hclen as usize {
        code_lengths[CODE_LENGTH_ALPHABET_INDICES[i]] = bits.read_bits(3, LSBFirst)? as u8;
    }
    bail!(
        "hlit = {}, hdist = {}, hclen = {}, code_lengths = {:?}",
        hlit,
        hdist,
        hclen,
        code_lengths
    );
}
