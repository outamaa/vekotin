use crate::fiddling::BitOrder::{LSBFirst, MSBFirst};
use crate::fiddling::BitStream;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::mem::zeroed;

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
    /// assert_eq!(alphabet.lookup(0b0000).unwrap(), 'F');
    /// assert_eq!(alphabet.lookup(0b0001).unwrap(), 'F');
    /// assert_eq!(alphabet.lookup(0b0010).unwrap(), 'F');
    /// assert_eq!(alphabet.lookup(0b0011).unwrap(), 'F');
    /// assert_eq!(alphabet.lookup(0b0100).unwrap(), 'A');
    /// assert_eq!(alphabet.lookup(0b1111).unwrap(), 'H');
    /// ```
    pub fn lookup(&self, code: u16) -> Option<S> {
        assert!(code <= self.max_lut_code);
        match self.lut[code as usize] {
            None => None,
            Some(tree_idx) => Some(self.tree[tree_idx].0),
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use vekotin::compression::deflate::HuffmanAlphabet;
    /// use vekotin::fiddling::BitStream;
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
    /// let encoded = [0b11110111u8, 0b10111000];
    /// let mut bits = BitStream::new(&encoded[..]);
    /// assert_eq!(alphabet.read_next(&mut bits).unwrap(), 'G');
    /// assert_eq!(alphabet.read_next(&mut bits).unwrap(), 'H');
    /// ```
    pub fn read_next<R: Read>(&self, bits: &mut BitStream<R>) -> Result<S> {
        let code = bits.peek_bits(self.max_code_length as usize, MSBFirst)? as u16;
        assert!(code <= self.max_lut_code);
        match self.lut[code as usize] {
            None => bail!("Couldn't find match in lut for code {:b}", code),
            Some(tree_idx) => {
                let entry = self.tree[tree_idx];
                bits.skip_bits(entry.1 as usize);
                Ok(self.tree[tree_idx].0)
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
    println!("cl_alphabet {:?}", cl_alphabet);

    let literal_alphabet = extract_alphabet(bits, hlit, &cl_alphabet)?;
    let distance_alphabet = extract_alphabet(bits, hdist, &cl_alphabet)?;

    Ok(())
}

fn extract_alphabet<R: Read>(
    bits: &mut BitStream<R>,
    alphabet_size: usize,
    cl_alphabet: &HuffmanAlphabet<u8>,
) -> Result<HuffmanAlphabet<u16>> {
    let mut literal_code_lengths = Vec::new();
    let mut cl_symbol: u16 = 0;
    println!("hlit = {}", alphabet_size);
    while (cl_symbol as usize) < alphabet_size {
        let s = cl_alphabet.read_next(bits)?;
        match s {
            0 => {
                println!("literal {}, length {}", cl_symbol, s);
                cl_symbol += 1;
            }
            1..=15 => {
                println!("literal {}, length {}", cl_symbol, s);
                literal_code_lengths.push((cl_symbol, s));
                cl_symbol += 1;
            }
            16 => {
                copy_last_length(bits, &mut literal_code_lengths, &mut cl_symbol)?;
            }
            17 => {
                repeat_zero(bits, &mut literal_code_lengths, &mut cl_symbol, 3, 3)?;
            }
            18 => {
                repeat_zero(bits, &mut literal_code_lengths, &mut cl_symbol, 7, 11)?;
            }
            _ => bail!("Invalid literal code length symbol: {}", s),
        }
    }
    // TODO: Either there's a bug in the above code or it's valid to tell to repeat zeros over
    // the alphabet size. Check that there are no more code lengths than given in header
    println!("cl_symbol at end {}", cl_symbol);

    Ok(HuffmanAlphabet::from_code_lengths(&literal_code_lengths))
}

fn copy_last_length<R: Read>(
    bits: &mut BitStream<R>,
    literal_code_lengths: &mut Vec<(u16, u8)>,
    cl_symbol: &mut u16,
) -> Result<()> {
    let copy_times = bits.read_bits(2, LSBFirst)? + 3;
    let last_code = literal_code_lengths.last();
    println!("repeat {:?} {} times", last_code, copy_times);
    match last_code {
        None => bail!("No last element in literal_code_lengths"),
        Some(&c) => {
            for _ in 0..copy_times {
                literal_code_lengths.push(c);
                *cl_symbol += 1;
            }
            Ok(())
        }
    }
}

fn repeat_zero<R: Read>(
    bits: &mut BitStream<R>,
    literal_code_lengths: &mut Vec<(u16, u8)>,
    cl_symbol: &mut u16,
    n_bits: usize,
    copy_start: u64,
) -> Result<()> {
    let zero_times = bits.read_bits(n_bits, LSBFirst)? + copy_start;
    println!("repeat zero {} times", zero_times);
    // add one to handle repeats
    literal_code_lengths.push((*cl_symbol, 0));
    *cl_symbol += zero_times as u16;

    Ok(())
}

enum DeflateSymbol {
    Literal(u8),
    LengthAndDistance(u16, u16),
    EndOfData,
}

fn read_deflate_symbol<R: Read>(
    bits: &mut BitStream<R>,
    literal_alphabet: &HuffmanAlphabet<u16>,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<DeflateSymbol> {
    use DeflateSymbol::*;

    let raw_symbol = literal_alphabet.read_next(bits)?;
    match raw_symbol {
        0..=255 => Ok(Literal(raw_symbol as u8)),
        256 => Ok(EndOfData),
        257..=285 => Ok(read_length_and_distance(
            bits,
            raw_symbol,
            distance_alphabet,
        )?),
        _ => bail!("Invalid Deflate symbol {}", raw_symbol),
    }
}

fn read_length_and_distance<R: Read>(
    bits: &mut BitStream<R>,
    length_symbol: u16,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<DeflateSymbol> {
    use DeflateSymbol::*;

    let symbol = match length_symbol {
        257..=264 => {
            let length = length_symbol - 254;
            let distance = read_distance(bits, distance_alphabet)?;
            LengthAndDistance(length, distance)
        }
        265..=268 => {
            read_length_and_distance_by_extra_bits(bits, length_symbol, 1, 265, distance_alphabet)?
        }
        269..=272 => {
            read_length_and_distance_by_extra_bits(bits, length_symbol, 2, 269, distance_alphabet)?
        }
        273..=276 => {
            read_length_and_distance_by_extra_bits(bits, length_symbol, 3, 273, distance_alphabet)?
        }
        _ => bail!("Invalide length symbol {}", length_symbol),
    };
    Ok(symbol)
}

fn read_length_and_distance_by_extra_bits<R: Read>(
    bits: &mut BitStream<R>,
    length_symbol: u16,
    extra_bits: usize,
    extra_bit_start: u16,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<DeflateSymbol> {
    use DeflateSymbol::*;

    let length = bits.read_bits(extra_bits, MSBFirst)? as u16
        + 3
        + 8 * extra_bits as u16
        + (length_symbol - extra_bit_start) * 2_u16.pow(extra_bits as u32);
    let distance = read_distance(bits, distance_alphabet)?;
    Ok(LengthAndDistance(length, distance))
}

fn read_distance<R: Read>(
    bits: &mut BitStream<R>,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<u16> {
    let raw_distance = distance_alphabet.read_next(bits)?;
    match raw_distance {
        _ => todo!(),
    }
}
