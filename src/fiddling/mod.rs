use std::io::Read;
use std::{cmp, io};

/// # Examples
///
/// ```rust
/// use vekotin::fiddling;
///
/// assert_eq!(0b11001111, fiddling::reverse_bits(0b11110011));
/// ```
pub fn reverse_bits(b: u8) -> u8 {
    let mut b = (b & 0b11110000) >> 4 | (b & 0b00001111) << 4;
    b = (b & 0b11001100) >> 2 | (b & 0b00110011) << 2;
    b = (b & 0b10101010) >> 1 | (b & 0b01010101) << 1;
    b
}

static FIRST_N_BITS: &'static [u8] = &[
    0b00000000, 0b00000001, 0b00000011, 0b00000111, 0b00001111, 0b00011111, 0b00111111, 0b01111111,
    0b11111111,
];

/// # Examples
///
/// ```rust
/// use vekotin::fiddling;
///
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
/// use vekotin::fiddling;
///
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
        byte >> 8 - n
    }
}

#[derive(PartialEq, Debug)]
pub enum BitOrder {
    MSBFirst,
    LSBFirst,
}

/// # Examples
///
/// ```rust
/// use vekotin::fiddling;
/// use vekotin::fiddling::BitOrder::*;
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 0, LSBFirst);
/// let b = 0b0101;
/// assert_eq!(a, b,
///            "Selecting bits from the start of a single byte, LSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 0, MSBFirst);
/// let b = 0b1010;
/// assert_eq!(a, b,
///            "Selecting bits from the start of a single byte, MSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 3, LSBFirst);
/// let b = 0b1010;
/// assert_eq!(a, b,
///            "Selecting bits from the middle of the byte, LSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 3, MSBFirst);
/// let b = 0b0101;
/// assert_eq!(a, b,
///            "Selecting bits from the middle of the byte, MSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 8, 6, LSBFirst);
/// let b = 0b11001101;
/// assert_eq!(a, b,
///            "Selecting bits from two contiguous bytes, LSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 8, 6, MSBFirst);
/// let b = 0b10110011;
/// assert_eq!(a, b,
///            "Selecting bits from two contiguous bytes, MSB first, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 0, 6, MSBFirst);
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
        if bit_order == MSBFirst {
            read_bits = last_n_bits(reverse_bits(read_bits as u8), n_bits_read as u64) as u64;
        }
        n -= n_bits_read;
        byte_idx = byte_idx + 1;
    }

    // Loop through whole bytes possibly truncating the final bits of the last one
    while n > 0 {
        let n_bits_read = cmp::min(n, 8);
        if bit_order == MSBFirst {
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
/// use vekotin::fiddling::BitStream;
/// use vekotin::fiddling::BitOrder::*;
///
/// let bytes: [u8; 2] = [0b01010101, 0b00110011];
/// let mut f = BitStream::new(&bytes[..]);
///
/// assert_eq!(f.peek_bits(0, MSBFirst).unwrap(), f.peek_bits(0, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(1, MSBFirst).unwrap(), f.peek_bits(1, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(4, MSBFirst).unwrap(), f.peek_bits(4, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(8, MSBFirst).unwrap(), f.peek_bits(8, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(9, MSBFirst).unwrap(), f.peek_bits(9, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(16, MSBFirst).unwrap(), f.peek_bits(16, MSBFirst).unwrap());
///
/// assert_eq!(f.peek_bits(0, LSBFirst).unwrap(), f.peek_bits(0, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(1, LSBFirst).unwrap(), f.peek_bits(1, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(4, LSBFirst).unwrap(), f.peek_bits(4, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(8, LSBFirst).unwrap(), f.peek_bits(8, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(9, LSBFirst).unwrap(), f.peek_bits(9, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(16, LSBFirst).unwrap(), f.peek_bits(16, LSBFirst).unwrap());
///
/// assert!(f.peek_bits(17, MSBFirst).err().is_some());
///
/// // Consume two bits
/// assert_eq!(f.peek_bits(2, MSBFirst).unwrap(), f.read_bits(2, MSBFirst).unwrap());
///
/// // Peeking should still work
/// assert_eq!(f.peek_bits(0, MSBFirst).unwrap(), f.peek_bits(0, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(1, MSBFirst).unwrap(), f.peek_bits(1, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(4, MSBFirst).unwrap(), f.peek_bits(4, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(8, MSBFirst).unwrap(), f.peek_bits(8, MSBFirst).unwrap());
/// assert_eq!(f.peek_bits(9, MSBFirst).unwrap(), f.peek_bits(9, MSBFirst).unwrap());
/// assert!(f.peek_bits(16, MSBFirst).err().is_some());
///
/// assert_eq!(f.peek_bits(0, LSBFirst).unwrap(), f.peek_bits(0, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(1, LSBFirst).unwrap(), f.peek_bits(1, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(4, LSBFirst).unwrap(), f.peek_bits(4, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(8, LSBFirst).unwrap(), f.peek_bits(8, LSBFirst).unwrap());
/// assert_eq!(f.peek_bits(9, LSBFirst).unwrap(), f.peek_bits(9, LSBFirst).unwrap());
/// assert!(f.peek_bits(16, LSBFirst).err().is_some());
///
/// f = BitStream::new(&bytes[..]);
/// assert_eq!(f.read_bits(3, LSBFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, LSBFirst).unwrap(), 0b010);
/// assert_eq!(f.read_bits(3, LSBFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, LSBFirst).unwrap(), 0b001);
/// assert_eq!(f.read_bits(3, LSBFirst).unwrap(), 0b011);
///
/// f = BitStream::new(&bytes[..]);
/// assert_eq!(f.read_bits(3, MSBFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, MSBFirst).unwrap(), 0b010);
/// assert_eq!(f.read_bits(3, MSBFirst).unwrap(), 0b101);
/// assert_eq!(f.read_bits(3, MSBFirst).unwrap(), 0b100);
/// assert_eq!(f.read_bits(3, MSBFirst).unwrap(), 0b110);
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
        self.read_bit_pos += n;
        Ok(result)
    }

    pub fn skip_bits(&mut self, n: usize) -> io::Result<()> {
        // TODO Might as well be possible to skip more bytes
        assert!(n <= (self.buf.len() - 1) * 8);
        self.ensure_readable_bits(n)?;
        self.read_bit_pos += n;
        Ok(())
    }

    pub fn read_u16_le(&mut self) -> io::Result<u16> {
        let buf = [self.read_next_byte()?, self.read_next_byte()?];
        Ok(u16::from_le_bytes(buf))
    }

    /// Skip to next byte boundary
    pub fn skip_to_next_byte(&mut self) -> io::Result<()> {
        self.skip_bits((8 - (self.read_bit_pos % 8)) as usize)?;
        Ok(())
    }

    /// If not at start of byte, skip to start of next one
    pub fn skip_to_start_of_byte(&mut self) -> io::Result<()> {
        if self.read_bit_pos % 8 != 0 {
            self.skip_to_next_byte()?;
        }
        Ok(())
    }

    /// Read next whole byte, skipping to the start of the next one if in the middle of the
    /// current one.
    pub fn read_next_byte(&mut self) -> io::Result<u8> {
        if !self.is_at_byte_boundary() {
            let _ = self.skip_to_next_byte()?;
        }
        self.ensure_readable_bits(8)?;
        let byte = self.buf[self.read_bit_pos / 8];
        self.read_bit_pos += 8;
        Ok(byte)
    }

    /// Load bytes from `inner` reader
    fn load_bytes(&mut self, n: usize) -> io::Result<()> {
        assert!(self.load_byte_pos + n <= self.buf.len());
        self.inner
            .read_exact(&mut self.buf[self.load_byte_pos..self.load_byte_pos + n])?;
        self.load_byte_pos += n;
        Ok(())
    }

    /// Rewind buffer so that read bytes are discarded and `read_bit_pos` resides in the first
    /// byte of `buf`.
    fn rewind_buffer(&mut self) {
        let read_byte_pos = self.read_bit_pos / 8;
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

    fn can_read_from_current_buf(&self, n: usize) -> bool {
        n <= self.readable_bits()
    }

    fn is_at_byte_boundary(&self) -> bool {
        self.read_bit_pos % 8 == 0
    }

    fn ensure_readable_bits(&mut self, n: usize) -> io::Result<()> {
        if !self.can_read_from_current_buf(n) {
            let bits_to_read = n - self.readable_bits();
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
