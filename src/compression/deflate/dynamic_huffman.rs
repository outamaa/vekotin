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
struct SymbolEntry<S: Copy + Ord> {
    symbol: S,
    length: u8,
    code: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HuffmanAlphabet<S: Copy + Ord> {
    tree: Vec<SymbolEntry<S>>,
    lut: Vec<Option<usize>>,
    max_lut_code: u16,
    max_code_length: u8,
}

impl<'a, S: 'a + Copy + Ord> HuffmanAlphabet<S> {
    pub fn from_code_lengths(code_lengths: &[(S, u8)]) -> Self {
        let max_code_length = *code_lengths
            .iter()
            .filter(|&(_, length)| *length > 0)
            .map(|(_, len)| len)
            .max()
            .unwrap();
        assert!(max_code_length < 16);
        let non_zero_code_lengths: Vec<(S, u8)> = code_lengths
            .iter()
            .filter(|&(_, length)| *length > 0)
            .cloned()
            .collect();
        let mut tree = <HuffmanAlphabet<S>>::assign_codes(&non_zero_code_lengths, max_code_length);

        // Build lookup table
        tree.sort_by(|a, b| a.code.cmp(&b.code));
        let mut lut: Vec<Option<usize>> = vec![None; 2usize.pow(max_code_length as u32)];

        for tree_idx in 0..tree.len() {
            let symbol_entry = &tree[tree_idx];
            let shift_by = max_code_length - symbol_entry.length;
            let lut_segment_start = (symbol_entry.code << shift_by) as usize;
            let lut_segment_end = ((symbol_entry.code + 1) << shift_by) as usize;
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
            Some(tree_idx) => Some(self.tree[tree_idx].symbol),
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
    /// assert_eq!(alphabet.read_next(&mut bits).unwrap(), 'F');
    /// assert_eq!(alphabet.read_next(&mut bits).unwrap(), 'B');
    /// ```
    pub fn read_next<R: Read>(&self, bits: &mut BitStream<R>) -> Result<S> {
        let code = bits.peek_bits(self.max_code_length as usize, MSBFirst)? as u16;
        assert!(code <= self.max_lut_code);
        match self.lut[code as usize] {
            None => bail!("Couldn't find match in lut for code {:b}", code),
            Some(tree_idx) => {
                let entry = &self.tree[tree_idx];
                bits.skip_bits(entry.length as usize);
                Ok(self.tree[tree_idx].symbol)
            }
        }
    }

    fn assign_codes(code_lengths: &Vec<(S, u8)>, max_code_length: u8) -> Vec<SymbolEntry<S>> {
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

        let mut tree: Vec<SymbolEntry<S>> = code_lengths
            .iter()
            .map(|&(s, len)| SymbolEntry {
                symbol: s,
                length: len,
                code: 0,
            })
            .collect();

        for n in 0..tree.len() {
            let len = tree[n].length;
            if len != 0 {
                tree[n].code = next_code[len as usize];
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

enum ExtractAction {
    CodeLength(u8),
    CopyLastLength(u8),
    RepeatZero(u8),
}

impl ExtractAction {
    fn from_bit_stream<R: Read>(
        bits: &mut BitStream<R>,
        alphabet: &HuffmanAlphabet<u8>,
    ) -> Result<ExtractAction> {
        use ExtractAction::*;
        let s = alphabet.read_next(bits)?;
        match s {
            0..=15 => Ok(CodeLength(s as u8)),
            16 => {
                let copy_times = bits.read_bits(2, LSBFirst)? + 3;
                Ok(CopyLastLength(copy_times as u8))
            }
            17 => {
                let zero_times = bits.read_bits(3, LSBFirst)? + 3;
                Ok(RepeatZero(zero_times as u8))
            }
            18 => {
                let zero_times = bits.read_bits(7, LSBFirst)? + 11;
                Ok(RepeatZero(zero_times as u8))
            }
            _ => bail!("Invalid literal code length symbol: {}", s),
        }
    }
}

pub fn extract_alphabet<R: Read>(
    bits: &mut BitStream<R>,
    alphabet_size: usize,
    cl_alphabet: &HuffmanAlphabet<u8>,
) -> Result<HuffmanAlphabet<u16>> {
    let mut literal_code_lengths = Vec::new();
    let mut cl_symbol: u16 = 0;
    println!("hlit = {}", alphabet_size);
    while (cl_symbol as usize) < alphabet_size {
        match ExtractAction::from_bit_stream(bits, cl_alphabet)? {
            ExtractAction::CodeLength(length) => {
                println!("code length {}", length);
                literal_code_lengths.push((cl_symbol, length));
                cl_symbol += 1;
            }
            ExtractAction::CopyLastLength(times) => {
                copy_last_length(times, &mut literal_code_lengths, &mut cl_symbol)?;
            }
            ExtractAction::RepeatZero(times) => {
                repeat_zero(times, &mut literal_code_lengths, &mut cl_symbol)?;
            }
        }
    }
    println!("cl_symbol at end {}", cl_symbol);

    Ok(HuffmanAlphabet::from_code_lengths(&literal_code_lengths))
}

fn copy_last_length(
    times: u8,
    literal_code_lengths: &mut Vec<(u16, u8)>,
    cl_symbol: &mut u16,
) -> Result<()> {
    let last_code = literal_code_lengths.last();
    println!("repeat {:?} {} times", last_code, times);
    match last_code {
        None => bail!("No last element in literal_code_lengths"),
        Some(&c) => {
            for _ in 0..times {
                literal_code_lengths.push(c);
                *cl_symbol += 1;
            }
            Ok(())
        }
    }
}

fn repeat_zero(
    times: u8,
    literal_code_lengths: &mut Vec<(u16, u8)>,
    cl_symbol: &mut u16,
) -> Result<()> {
    println!("repeat zero {} times", times);
    // add one to handle repeats
    literal_code_lengths.push((*cl_symbol, 0));
    *cl_symbol += times as u16;

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

static LENGTH_EXTRA_BITS: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

static BASE_LENGTH: [u8; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];

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
        _ => bail!("Invalid length symbol {}", length_symbol),
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

static DISTANCE_EXTRA_BITS: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

static BASE_DISTANCE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];

fn read_distance<R: Read>(
    bits: &mut BitStream<R>,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<u16> {
    let raw_distance = distance_alphabet.read_next(bits)?;
    match raw_distance {
        0..=3 => Ok(raw_distance + 1),
        _ => {
            let extra_bits = DISTANCE_EXTRA_BITS[raw_distance as usize];
            let base_distance = BASE_DISTANCE[raw_distance as usize];
            Ok(base_distance + bits.read_bits(extra_bits as usize, MSBFirst)? as u16)
        }
    }
}
