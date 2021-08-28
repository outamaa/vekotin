use crate::fiddling::BitOrder::{LSBFirst, MSBFirst};
use crate::fiddling::BitStream;
use anyhow::{bail, Error, Result};
use lazy_static::lazy_static;
use std::io::{Read, Write};
use std::iter;

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

lazy_static! {
    pub static ref STATIC_DISTANCE_ALPHABET: HuffmanAlphabet<u16> = {
        let code_lengths: Vec<(u16, u8)> = (0u16..32).zip(iter::repeat(5u8)).collect();
        HuffmanAlphabet::from_code_lengths(&code_lengths[..])
    };
    pub static ref STATIC_LITERAL_ALPHABET: HuffmanAlphabet<u16> = {
        let code_lengths: Vec<(u16, u8)> = (0u16..144)
            .zip(iter::repeat(8u8))
            .chain((144..256).zip(iter::repeat(9)))
            .chain((256..280).zip(iter::repeat(7)))
            .chain((280..288).zip(iter::repeat(8)))
            .collect();
        HuffmanAlphabet::from_code_lengths(&code_lengths[..])
    };
}

impl<'a, S: 'a + Copy + Ord> HuffmanAlphabet<S> {
    pub fn from_code_lengths(code_lengths: &[(S, u8)]) -> HuffmanAlphabet<S> {
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
        let mut tree = Self::assign_codes(&non_zero_code_lengths, max_code_length);

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

        Self {
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

pub fn copy_dynamic_huffman_block<R: Read>(
    bits: &mut BitStream<R>,
    out_buf: &mut Vec<u8>,
) -> Result<()> {
    let hlit = (bits.read_bits(5, LSBFirst)? + 257) as usize;
    assert!(257 <= hlit && hlit <= 286);
    let hdist = (bits.read_bits(5, LSBFirst)? + 1) as usize;
    assert!(1 <= hdist && hdist <= 32);
    let hclen = (bits.read_bits(4, LSBFirst)? + 4) as usize;
    assert!(4 <= hclen && hclen <= 19);

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

    copy_huffman_block(bits, out_buf, &literal_alphabet, &distance_alphabet)
}

pub fn copy_static_huffman_block<R: Read>(
    bits: &mut BitStream<R>,
    out_buf: &mut Vec<u8>,
) -> Result<()> {
    copy_huffman_block(
        bits,
        out_buf,
        &STATIC_LITERAL_ALPHABET,
        &STATIC_DISTANCE_ALPHABET,
    )
}

fn copy_huffman_block<R: Read>(
    bits: &mut BitStream<R>,
    out_buf: &mut Vec<u8>,
    literal_alphabet: &HuffmanAlphabet<u16>,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<(), Error> {
    let mut total = 0;
    while let Ok(symbol) = read_deflate_symbol(bits, &literal_alphabet, &distance_alphabet) {
        use DeflateSymbol::*;
        match symbol {
            Literal(value) => {
                out_buf.push(value);
                total += 1;
            }
            LengthAndDistance(length, distance) => {
                let current_idx = out_buf.len();
                assert!(
                    distance as usize <= current_idx,
                    "length={}, distance {} <= current_idx {}",
                    length,
                    distance,
                    current_idx
                );
                let copy_start = current_idx - distance as usize;
                let copy_end = copy_start + length as usize;
                for idx in copy_start..copy_end {
                    out_buf.push(out_buf[idx]);
                    total += 1;
                }
            }
            EndOfData => {
                break;
            }
        }
        println!("{:?}, total={}", symbol, total);
    }
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
    match last_code {
        None => bail!("No last element in literal_code_lengths"),
        Some(&(symbol, length)) => {
            for _ in 0..times {
                literal_code_lengths.push((*cl_symbol, length));
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
    // add one to handle repeats
    literal_code_lengths.push((*cl_symbol, 0));
    *cl_symbol += times as u16;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    0, 0, 0, 0, 0, 0, 0, 0, // 257 - 264
    1, 1, 1, 1, //             265 - 268
    2, 2, 2, 2, //             269 - 272
    3, 3, 3, 3, //             273 - 276
    4, 4, 4, 4, //             277 - 280
    5, 5, 5, 5, //             280 - 284
    0, //                      285
];

static BASE_LENGTH: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, // 0 extra bits
    11, 13, 15, 17, //          1 extra bits
    19, 23, 27, 31, //          2 extra bits
    35, 43, 51, 59, //          3 extra bits
    67, 83, 99, 115, //         4 extra bits
    131, 163, 195, 227, //      5 extra bits
    258, //                     0 extra bits
];

fn read_length_and_distance<R: Read>(
    bits: &mut BitStream<R>,
    length_symbol: u16,
    distance_alphabet: &HuffmanAlphabet<u16>,
) -> Result<DeflateSymbol> {
    use DeflateSymbol::*;

    // TODO look this through
    let extra_bits = LENGTH_EXTRA_BITS[(length_symbol - 257) as usize];
    let base_length = BASE_LENGTH[(length_symbol - 257) as usize];
    let length = base_length + bits.read_bits(extra_bits as usize, MSBFirst)? as u16;

    let distance = read_distance(bits, distance_alphabet)?;

    Ok(LengthAndDistance(length, distance))
}

static DISTANCE_EXTRA_BITS: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

static BASE_DISTANCE: [u16; 30] = [
    1, 2, 3, 4, //   0 extra bits
    5, 7, //         1 extra bit
    9, 13, //        2 extra bits
    17, 25, //       3 extra bits
    33, 49, //       4 extra bits
    65, 97, //       5 extra bits
    129, 193, //     6 extra bits
    257, 385, //     7 extra bits
    513, 769, //     8 extra bits
    1025, 1537, //   9 extra bits
    2049, 3073, //   10 extra bits
    4097, 6145, //   11 extra bits
    8193, 12289, //  12 extra bits
    16385, 24577, // 13 extra bits
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_deflate_symbol() {
        use DeflateSymbol::*;
        let alphabet = HuffmanAlphabet::static_literal_alphabet();

        let bytes = [0b00001100, 0xaa];
        assert_symbol(Literal(0), &alphabet, &bytes);

        let bytes = [0b11111111, 0b1];
        assert_symbol(Literal(255), &alphabet, &bytes);

        let bytes = [0b0, 0b0];
        assert_symbol(EndOfData, &alphabet, &bytes);

        // Length  Distance
        // 257=3   0=1
        // 0000001 0|0110000
        let bytes = [0b01000000, 0b0000110];
        assert_symbol(LengthAndDistance(3, 1), &alphabet, &bytes[..]);
        // Length   Extra   Distance  Extra
        // 280       14=129 6         3=12
        // 11000000 |1110   0011|0110 11 (00)
        let bytes = [0b00000011, 0b11000111, 0b00110110];
        assert_symbol(LengthAndDistance(129, 12), &alphabet, &bytes[..]);
    }

    #[test]
    fn test_read_distance() {
        let distance_alphabet = HuffmanAlphabet::static_literal_alphabet();

        let bytes = [0b00001100u8, 0xaa];
        assert_distance(1, &distance_alphabet, &bytes);

        let bytes = [0b10001100u8, 0xaa];
        assert_distance(2, &distance_alphabet, &bytes);

        let bytes = [0b00101100u8, 0b0];
        assert_distance(5, &distance_alphabet, &bytes);

        let bytes = [0b00101100u8, 0b1];
        assert_distance(6, &distance_alphabet, &bytes);

        let bytes = [0b01101100u8, 0b00];
        assert_distance(9, &distance_alphabet, &bytes);

        let bytes = [0b01101100u8, 0b10];
        assert_distance(10, &distance_alphabet, &bytes);

        let bytes = [0b01101100u8, 0b01];
        assert_distance(11, &distance_alphabet, &bytes);

        let bytes = [0b01101100u8, 0b11];
        assert_distance(12, &distance_alphabet, &bytes);
    }

    #[test]
    fn static_literal() {
        let code_lengths: Vec<(u16, u8)> = (0u16..144)
            .zip(iter::repeat(8u8))
            .chain((144..256).zip(iter::repeat(9)))
            .chain((256..280).zip(iter::repeat(7)))
            .chain((280..288).zip(iter::repeat(8)))
            .collect();
        let test = HuffmanAlphabet::from_code_lengths(&code_lengths[..]);
        assert_eq!(test, HuffmanAlphabet::static_literal_alphabet());
    }

    fn assert_distance(
        expected_distance: u16,
        distance_alphabet: &HuffmanAlphabet<u16>,
        bytes: &[u8; 2],
    ) {
        let mut bits = BitStream::new(&bytes[..]);
        let distance = read_distance(&mut bits, &distance_alphabet);
        assert_eq!(expected_distance, distance.unwrap());
    }

    fn assert_symbol(
        expected_symbol: DeflateSymbol,
        alphabet: &HuffmanAlphabet<u16>,
        bytes: &[u8],
    ) {
        let mut bits = BitStream::new(&bytes[..]);
        let symbol = read_deflate_symbol(&mut bits, &alphabet, &alphabet);
        assert_eq!(expected_symbol, symbol.unwrap());
    }
}
