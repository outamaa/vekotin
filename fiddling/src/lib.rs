use std::io::Read;
use std::{cmp, io};

/// # Examples
///
/// ```rust
/// assert_eq!(0b11001111, fiddling::reverse_bits(0b11110011));
/// ```
pub fn reverse_bits(b: u8) -> u8 {
    let mut b = (b & 0b11110000) >> 4 | (b & 0b00001111) << 4;
    b = (b & 0b11001100) >> 2 | (b & 0b00110011) << 2;
    b = (b & 0b10101010) >> 1 | (b & 0b01010101) << 1;
    b
}

static FIRST_N_BITS: [u8; 9] = [
    0b00000000, 0b00000001, 0b00000011, 0b00000111, 0b00001111, 0b00011111, 0b00111111, 0b01111111,
    0b11111111,
];

/// # Examples
///
/// ```rust
/// let bits = fiddling::first_n_bits(0b11111111, 0);
/// assert_eq!(bits, 0);
///
/// let bits = fiddling::first_n_bits(0b11111111, 3);
/// assert_eq!(bits, 0b00000111);
///
/// assert_eq!(fiddling::first_n_bits(0b11111111, 100),
///            fiddling::first_n_bits(0b11111111, 8));
/// ```
pub fn first_n_bits(byte: u8, n: u64) -> u8 {
    byte & FIRST_N_BITS[cmp::min(n, 8) as usize]
}

/// # Examples
///
/// ```rust
/// let bits = fiddling::last_n_bits(0b11111111, 0);
/// assert_eq!(bits, 0);
///
/// let bits = fiddling::last_n_bits(0b10111111, 3);
/// assert_eq!(bits, 0b00000101);
///
/// assert_eq!(fiddling::last_n_bits(0b11111111, 100),
///            fiddling::last_n_bits(0b11111111, 8));
/// ```
pub fn last_n_bits(byte: u8, n: u64) -> u8 {
    if n >= 8 {
        byte
    } else if n == 0 {
        0
    } else {
        byte >> (8 - n)
    }
}

#[derive(PartialEq, Debug)]
pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

/// # Examples
///
/// ```rust
/// use fiddling::BitOrder::*;
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 0, LsbFirst);
/// let b = 0b0101;
/// assert_eq!(a, b,
///            "Selecting bits from the start of a single byte, LSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 0, MsbFirst);
/// let b = 0b1010;
/// assert_eq!(a, b,
///            "Selecting bits from the start of a single byte, MSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 3, LsbFirst);
/// let b = 0b1010;
/// assert_eq!(a, b,
///            "Selecting bits from the middle of the byte, LSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 3, MsbFirst);
/// let b = 0b0101;
/// assert_eq!(a, b,
///            "Selecting bits from the middle of the byte, MSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 8, 6, LsbFirst);
/// let b = 0b11001101;
/// assert_eq!(a, b,
///            "Selecting bits from two contiguous bytes, LSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 8, 6, MsbFirst);
/// let b = 0b10110011;
/// assert_eq!(a, b,
///            "Selecting bits from two contiguous bytes, MSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 0, 6, MsbFirst);
/// let b = 0;
/// assert_eq!(a, b,
///            "Selecting 0 bits returns zero");

/// ```
pub fn n_bits_by_index(bytes: &[u8], n_bits: u8, bit_idx: usize, bit_order: BitOrder) -> u64 {
    use BitOrder::*;
    assert!(n_bits <= 64);

    let original_byte_idx = bit_idx / 8;
    let mut byte_idx = original_byte_idx;

    let within_byte_idx: u8 = (bit_idx % 8) as u8;
    let mut n = n_bits;
    let mut read_bits: u64 = 0;

    // If we start from the middle of a byte
    if within_byte_idx != 0 {
        let last_n = 8 - within_byte_idx;
        let n_bits_read = cmp::min(last_n, n);
        read_bits = first_n_bits(last_n_bits(bytes[byte_idx], last_n as u64), n as u64) as u64;
        if bit_order == MsbFirst {
            read_bits = last_n_bits(reverse_bits(read_bits as u8), n_bits_read as u64) as u64;
        }
        n -= n_bits_read;
        byte_idx = byte_idx + 1;
    }

    // Loop through whole bytes possibly truncating the final bits of the last one
    while n > 0 {
        let n_bits_read = cmp::min(n, 8);
        if bit_order == MsbFirst {
            read_bits = read_bits << n_bits_read
                | last_n_bits(reverse_bits(bytes[byte_idx]), n_bits_read as u64) as u64;
        } else {
            read_bits = read_bits
                + ((first_n_bits(bytes[byte_idx], n_bits_read as u64) as u64) << (n_bits - n));
        }

        n -= n_bits_read;
        byte_idx = byte_idx + 1;
    }
    read_bits
}

/// # Examples
///
/// ```rust
/// use fiddling::BitStream;
/// use fiddling::BitOrder::*;
///
/// let bytes: [u8; 2] = [0b01010101, 0b00110011];
/// let mut f = BitStream::new(&bytes[..]);
///
/// assert_eq!(f.peek_bits(0, MsbFirst).unwrap(), f.peek_bits(0, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(1, MsbFirst).unwrap(), f.peek_bits(1, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(4, MsbFirst).unwrap(), f.peek_bits(4, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(8, MsbFirst).unwrap(), f.peek_bits(8, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(9, MsbFirst).unwrap(), f.peek_bits(9, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(16, MsbFirst).unwrap(), f.peek_bits(16, MsbFirst).unwrap());
///
/// assert_eq!(f.peek_bits(0, LsbFirst).unwrap(), f.peek_bits(0, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(1, LsbFirst).unwrap(), f.peek_bits(1, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(4, LsbFirst).unwrap(), f.peek_bits(4, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(8, LsbFirst).unwrap(), f.peek_bits(8, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(9, LsbFirst).unwrap(), f.peek_bits(9, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(16, LsbFirst).unwrap(), f.peek_bits(16, LsbFirst).unwrap());
///
/// assert!(f.peek_bits(17, MsbFirst).err().is_some());
///
/// // Consume two bits
/// assert_eq!(f.peek_bits(2, MsbFirst).unwrap(), f.read_bits(2, MsbFirst).unwrap());
///
/// // Peeking should still work
/// assert_eq!(f.peek_bits(0, MsbFirst).unwrap(), f.peek_bits(0, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(1, MsbFirst).unwrap(), f.peek_bits(1, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(4, MsbFirst).unwrap(), f.peek_bits(4, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(8, MsbFirst).unwrap(), f.peek_bits(8, MsbFirst).unwrap());
/// assert_eq!(f.peek_bits(9, MsbFirst).unwrap(), f.peek_bits(9, MsbFirst).unwrap());
/// assert!(f.peek_bits(16, MsbFirst).err().is_some());
///
/// assert_eq!(f.peek_bits(0, LsbFirst).unwrap(), f.peek_bits(0, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(1, LsbFirst).unwrap(), f.peek_bits(1, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(4, LsbFirst).unwrap(), f.peek_bits(4, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(8, LsbFirst).unwrap(), f.peek_bits(8, LsbFirst).unwrap());
/// assert_eq!(f.peek_bits(9, LsbFirst).unwrap(), f.peek_bits(9, LsbFirst).unwrap());
/// assert!(f.peek_bits(16, LsbFirst).err().is_some());
///
/// f = BitStream::new(&bytes[..]);
/// assert_eq!(f.read_bits(3, LsbFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, LsbFirst).unwrap(), 0b010);
/// assert_eq!(f.read_bits(3, LsbFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, LsbFirst).unwrap(), 0b001);
/// assert_eq!(f.read_bits(3, LsbFirst).unwrap(), 0b011);
///
/// f = BitStream::new(&bytes[..]);
/// assert_eq!(f.read_bits(3, MsbFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, MsbFirst).unwrap(), 0b010);
/// assert_eq!(f.read_bits(3, MsbFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, MsbFirst).unwrap(), 0b100);
/// assert_eq!(f.read_bits(3, MsbFirst).unwrap(), 0b110);
/// ```
pub struct BitStream<R> {
    inner: R,
    buf: [u8; 5], // 64 bits (ought to be enough for everybody) + one extra byte
    read_bit_pos: usize,
    load_byte_pos: usize,
}
/// A reader for reading a byte stream on a bit basis,
impl<R: Read> BitStream<R> {
    pub fn new(inner: R) -> BitStream<R> {
        BitStream {
            inner,
            buf: [0; 5],
            read_bit_pos: 0,
            load_byte_pos: 0,
        }
    }

    /// Peek at the next `n` bits. Does not change the bit position of fiddler, but _can_ read more
    /// bytes from the `inner` reader.
    pub fn peek_bits(&mut self, n: usize, bo: BitOrder) -> io::Result<u64> {
        assert!(n <= (self.buf.len() - 1) * 8);
        self.ensure_readable_bits(n)?;
        Ok(n_bits_by_index(&self.buf, n as u8, self.read_bit_pos, bo))
    }

    /// Read (and consume) the next `n` bits from the `inner` reader.
    pub fn read_bits(&mut self, n: usize, bo: BitOrder) -> io::Result<u64> {
        let result = self.peek_bits(n, bo)?;
        self.skip_bits(n);
        Ok(result)
    }

    pub fn skip_bits(&mut self, n: usize) {
        // TODO Might as well be possible to skip more bytes
        assert!(n <= (self.buf.len() - 1) * 8);
        self.read_bit_pos += n;
    }

    pub fn read_u16_le(&mut self) -> io::Result<u16> {
        let buf = [self.read_next_byte()?, self.read_next_byte()?];
        Ok(u16::from_le_bytes(buf))
    }

    /// Skip to next byte boundary
    pub fn skip_to_next_byte(&mut self) {
        self.skip_bits((8 - (self.read_bit_pos % 8)) as usize);
    }

    /// If not at start of byte, skip to start of next one
    pub fn skip_to_start_of_byte(&mut self) {
        if !self.is_at_byte_boundary() {
            self.skip_to_next_byte();
        }
    }

    /// Read next whole byte, skipping to the start of the next one if in the middle of the
    /// current one.
    pub fn read_next_byte(&mut self) -> io::Result<u8> {
        self.skip_to_start_of_byte();
        self.ensure_readable_bits(8)?;
        let byte = self.buf[self.read_byte_pos()];
        self.read_bit_pos += 8;
        Ok(byte)
    }

    fn read_byte_pos(&mut self) -> usize {
        self.read_bit_pos / 8
    }

    /// Load bytes from `inner` reader
    fn load_bytes(&mut self, n_bytes: usize) -> io::Result<()> {
        assert!(self.load_byte_pos + n_bytes <= self.buf.len());
        self.inner
            .read_exact(&mut self.buf[self.load_byte_pos..self.load_byte_pos + n_bytes])?;
        self.load_byte_pos += n_bytes;
        Ok(())
    }

    /// Rewind buffer so that read bytes are discarded and `read_bit_pos` resides in the first
    /// byte of `buf`.
    fn rewind_buffer(&mut self) {
        let read_byte_pos = self.read_byte_pos();
        if read_byte_pos == 0 {
            println!("called rewind_buffer with read_byte_pos = 0");
            return;
        }
        if read_byte_pos < self.load_byte_pos {
            for i in read_byte_pos..self.load_byte_pos {
                self.buf[i - read_byte_pos] = self.buf[i];
            }
        }
        self.read_bit_pos %= 8;
        self.load_byte_pos -= read_byte_pos;
    }

    fn reset(&mut self) {
        self.load_byte_pos = 0;
        self.read_bit_pos = 0;
    }
    fn readable_bits(&self) -> usize {
        8 * self.load_byte_pos - self.read_bit_pos
    }

    fn loadable_bits(&self) -> usize {
        8 * (self.buf.len() - self.load_byte_pos)
    }

    fn can_read_from_current_buf(&self, n_bits: usize) -> bool {
        n_bits <= self.readable_bits()
    }

    fn is_at_byte_boundary(&self) -> bool {
        self.read_bit_pos % 8 == 0
    }

    fn ensure_readable_bits(&mut self, n_bits: usize) -> io::Result<()> {
        if !self.can_read_from_current_buf(n_bits) {
            let bits_to_read = n_bits - self.readable_bits();
            if self.loadable_bits() < bits_to_read {
                self.rewind_buffer();
            }
            self.load_bytes((bits_to_read + 7) / 8)?;
        }
        Ok(())
    }
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.reset();
        &mut self.inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::BitOrder::{LsbFirst, MsbFirst};
    use super::*;

    #[test]
    fn test_multiple_reads() {
        let bytes: [u8; 12] = [
            0b0000_0001,
            0b0010_0011,
            0b0100_0101,
            0b0110_0111,
            0b1000_1001,
            0b1010_1011,
            0b1100_1101,
            0b1110_1111, // read_u16_le
            0b1111_1111, //
            0b1010_1011, // read_bits
            0b1100_1101, // read_u16_le
            0b1110_1111,
        ];
        let mut f = BitStream::new(&bytes[..]);
        assert_eq!(f.read_bits(3, MsbFirst).unwrap(), 0b100);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(4, LsbFirst).unwrap(), 0b0000);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(0, LsbFirst).unwrap(), 0);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(8, MsbFirst).unwrap(), 0b0110_0010);
        f.peek_bits(9, MsbFirst).unwrap();
        assert!(!f.is_at_byte_boundary());
        f.peek_bits(9, MsbFirst).unwrap();
        f.skip_to_next_byte();
        f.peek_bits(9, MsbFirst).unwrap();
        assert!(f.is_at_byte_boundary());
        f.peek_bits(9, MsbFirst).unwrap();
        f.skip_to_start_of_byte(); // Should be no-op here
        f.peek_bits(9, MsbFirst).unwrap();
        assert!(f.is_at_byte_boundary());
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.peek_bits(5, LsbFirst).unwrap(), 0b0_0101);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(5, LsbFirst).unwrap(), 0b0_0101);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(0, MsbFirst).unwrap(), 0);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(5, LsbFirst).unwrap(), 0b1_1010);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(5, MsbFirst).unwrap(), 0b1_0011);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.peek_bits(10, MsbFirst).unwrap(), 0b01_0010_0011);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(10, MsbFirst).unwrap(), 0b01_0010_0011);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_next_byte().unwrap(), 0b1100_1101);
        f.peek_bits(9, MsbFirst).unwrap();
        assert!(f.is_at_byte_boundary());
        assert_eq!(f.read_u16_le().unwrap(), 0b1111_1111_1110_1111);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(0, LsbFirst).unwrap(), 0);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(1, LsbFirst).unwrap(), 1);
        f.peek_bits(9, MsbFirst).unwrap();
        assert_eq!(f.read_bits(1, MsbFirst).unwrap(), 1);
        // Skips to the start of next byte
        assert_eq!(f.read_u16_le().unwrap(), 0b1110_1111_1100_1101);
    }
}
