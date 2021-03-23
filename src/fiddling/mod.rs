use std::cmp;

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
/// let b = 0b0000101;
/// assert_eq!(a, b,
///            "Selecting bits from the start of a single byte, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101], 4, 3, LSBFirst);
/// let b = 0b0001010;
/// assert_eq!(a, b,
///            "Selecting bits from the middle of the byte, {:b} == {:b}", a, b);
///
/// let a = fiddling::n_bits_by_index(&[0b01010101, 0b00110011], 8, 6, LSBFirst);
/// let b = 0b11001101;
/// assert_eq!(a, b,
///            "Selecting bits from two contiguous bytes, {:b} == {:b}", a, b);
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

        if bit_order == MSBFirst {
            read_bits = first_n_bits(reverse_bits(bytes[byte_idx]), last_n as u64) as u64;
        } else {
            read_bits = first_n_bits(last_n_bits(bytes[byte_idx], last_n as u64), n as u64).into();
        }
        n = n - cmp::min(last_n, n);
        byte_idx = byte_idx + 1;
    }

    // Loop through whole bytes possibly truncating the final bits of the last one
    while n > 0 {
        if bit_order == MSBFirst {
            read_bits =
                read_bits * 256 + last_n_bits(reverse_bits(bytes[byte_idx]), n as u64) as u64;
        } else {
            let shift_bits = n_bits - n;
            read_bits =
                read_bits + ((first_n_bits(bytes[byte_idx], n as u64) as u64) << shift_bits);
        }

        n = cmp::max(n, 8) - 8;
        if n == 0 {
            break;
        }
        byte_idx = byte_idx + 1;
    }
    read_bits
}

struct Fiddler {}
