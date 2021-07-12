use crate::fiddling::BitOrder::{LSBFirst, MSBFirst};
use crate::fiddling::BitStream;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::{Read, Write};

const CODE_LENGTH_ALPHABET_INDICES: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Clone, Debug, PartialEq)]
pub struct HuffmanAlphabet<S: Copy + Ord> {
    tree: Vec<(S, u8, u16)>,
    lut: Vec<Option<usize>>,
    max_lut_code: u16,
    max_code_length: u8,
}

impl<S: Copy + Ord> HuffmanAlphabet<S> {
    pub fn from_code_lengths(code_lengths: &[(S, u8)]) -> Self {
        let max_code_length = *code_lengths.iter().map(|(_, len)| len).max().unwrap();
        assert!(max_code_length < 16);
        let mut tree = <HuffmanAlphabet<S>>::assign_codes(code_lengths, max_code_length);

        // Build lookup table
        tree.sort_by(|a, b| a.2.cmp(&b.2));
        let mut lut: Vec<Option<usize>> = vec![None; 2usize.pow(max_code_length as u32)];

        for tree_idx in 0..tree.len() {
            let symbol_entry = tree[tree_idx];
            let shift_by = max_code_length - symbol_entry.1;
            let lut_segment_start = (symbol_entry.2 << shift_by) as usize;
            let lut_segment_end = ((symbol_entry.2 + 1) << shift_by) as usize;
            for lut_idx in lut_segment_start..lut_segment_end {
                lut[lut_idx] = Some(tree_idx);
            }
        }

        HuffmanAlphabet {
            tree,
            lut,
            max_lut_code: (1 << max_code_length) - 1,
            max_code_length,
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::compression::deflate::HuffmanAlphabet;
    /// // Example from PNG RFC:
    /// // Symbol Length   Code
    /// // ------ ------   ----
    /// // A       3        010
    /// // B       3        011
    /// // C       3        100
    /// // D       3        101
    /// // E       3        110
    /// // F       2         00
    /// // G       4       1110
    /// // H       4       1111
    /// let code_lengths = [('A', 3u8), ('B', 3), ('C', 3), ('D', 3), ('E', 3), ('F', 2), ('G', 4), ('H', 4)];
    ///
    /// let alphabet = HuffmanAlphabet::from_code_lengths(&code_lengths[..]);
    /// assert_eq!(alphabet.lookup(0b0000), Some('F'));
    /// assert_eq!(alphabet.lookup(0b0001), Some('F'));
    /// assert_eq!(alphabet.lookup(0b0010), Some('F'));
    /// assert_eq!(alphabet.lookup(0b0011), Some('F'));
    /// assert_eq!(alphabet.lookup(0b0100), Some('A'));
    /// assert_eq!(alphabet.lookup(0b1111), Some('H'));
    /// ```
    pub fn lookup(&self, code: u16) -> Option<S> {
        assert!(code <= self.max_lut_code);
        match self.lut[code as usize] {
            None => None,
            Some(tree_idx) => Some(self.tree[tree_idx].0),
        }
    }

    pub fn read_next<R: Read>(&self, bits: &mut BitStream<R>) -> Result<Option<S>> {
        let code = bits.peek_bits(self.max_code_length as usize, MSBFirst)? as u16;
        assert!(code <= self.max_lut_code);
        match self.lut[code as usize] {
            None => Ok(None),
            Some(tree_idx) => {
                let entry = self.tree[tree_idx];
                bits.skip_bits(entry.1 as usize);
                Ok(Some(self.tree[tree_idx].0))
            }
        }
    }

    fn assign_codes(code_lengths: &[(S, u8)], max_code_length: u8) -> Vec<(S, u8, u16)> {
        let mut bl_count = vec![0; max_code_length as usize + 1];
        code_lengths.iter().for_each(|&(_, x)| {
            bl_count[x as usize] += 1;
        });
        bl_count[0] = 0;

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
        tree
    }
}

pub fn copy_dynamic_huffman_block<R: Read, W: Write>(
    bits: &mut BitStream<R>,
    out_bytes: &mut W,
) -> Result<()> {
    let hlit = (bits.read_bits(5, LSBFirst)? + 257) as usize;
    let hdist = (bits.read_bits(5, LSBFirst)? + 1) as usize;
    let hclen = (bits.read_bits(4, LSBFirst)? + 4) as usize;

    let mut code_lengths = vec![(0u8, 0u8); 19];
    for i in 0..hclen {
        code_lengths[CODE_LENGTH_ALPHABET_INDICES[i]] = (
            CODE_LENGTH_ALPHABET_INDICES[i] as u8,
            bits.read_bits(3, LSBFirst)? as u8,
        );
    }

    let cl_alphabet = HuffmanAlphabet::from_code_lengths(&code_lengths);

    // Next up: Literal code lenghts
    // Remember repeats on 16, 17, 18!

    bail!(
        "hlit = {}, hdist = {}, hclen = {}, code_lengths = {:?}, literal_code_lengths = {:?}",
        hlit,
        hdist,
        hclen,
        code_lengths,
        literal_code_lengths,
    );
}
