use std::io;
use std::io::Read;

//
// Digest trait
//

pub trait Digest {
    fn update(&mut self, b: u8);
    fn digest(&self) -> u32;
    fn reset(&mut self);
}

//
// CRC-32
//

const fn make_crc_table() -> [u32; 256] {
    let mut n: usize = 0;
    let mut crc_table: [u32; 256] = [0; 256];
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1)
            } else {
                c = c >> 1;
            }
            crc_table[n] = c;
            k += 1;
        }
        n += 1;
    }
    crc_table
}

const CRC_TABLE: [u32; 256] = make_crc_table();

pub struct Crc32 {
    crc: u32,
}

/// # Examples
///
/// ```rust
/// use digest::{Crc32, Digest};
///
/// let mut crc = Crc32::new();
/// let data = [0x49 as u8, 0x48, 0x44, 0x52, 0x00, 0x00, 0x03, 0x20, 0x00, 0x00, 0x02, 0x58,
/// 0x08, 0x06, 0x00, 0x00, 0x00];
///
/// for b in &data {
///   crc.update(*b);
/// }
///
/// assert_eq!(crc.digest(), 2591457904);
/// ```
impl Crc32 {
    pub fn new() -> Crc32 {
        Crc32 { crc: 0xffffffff }
    }
}

impl Digest for Crc32 {
    fn update(&mut self, b: u8) {
        self.crc = CRC_TABLE[((self.crc ^ b as u32) & 0xff) as usize] ^ (self.crc >> 8);
    }

    fn digest(&self) -> u32 {
        self.crc ^ 0xffffffff
    }

    fn reset(&mut self) {
        self.crc = 0xffffffff;
    }
}

/// # Examples
///
/// ```rust
/// use std::io::Read;
/// use digest::{DigestReader, Crc32};
///
/// let input = vec![0x49 as u8, 0x48, 0x44, 0x52, 0x00, 0x00, 0x03, 0x20, 0x00, 0x00, 0x02, 0x58,
/// 0x08, 0x06, 0x00, 0x00, 0x00];
/// let mut reader = DigestReader::new(input.as_slice(), Crc32::new());
///
/// let mut output = vec![0 as u8; 17];
///
/// reader.read(&mut output).unwrap();
///
/// assert_eq!(input, output);
/// assert_eq!(reader.digest(), 2591457904);
/// ```
pub struct DigestReader<R, D> {
    inner: R,
    digest: D,
}

impl<R: Read, D: Digest> DigestReader<R, D> {
    pub fn new(inner: R, digest: D) -> DigestReader<R, D> {
        DigestReader { inner, digest }
    }

    pub fn digest(&self) -> u32 {
        self.digest.digest()
    }

    pub fn reset_digest(&mut self) {
        self.digest.reset();
    }
}

impl<R, D> DigestReader<R, D> {
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read, D: Digest> Read for DigestReader<R, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n_read = self.inner.read(buf)?;
        for b in buf.iter().take(n_read) {
            self.digest.update(*b);
        }
        Ok(n_read)
    }
}

//
// Adler-32
//

pub struct Adler32 {
    a: u16,
    b: u16,
}

/// # Examples
///
/// ```rust
/// // Test uses the example presented in https://en.wikipedia.org/wiki/Adler-32
/// use digest::{Adler32, Digest};
///
/// let mut adler = Adler32::new();
/// let data = [87 as u8, 105, 107, 105, 112, 101, 100, 105, 97,];
///
/// for b in &data {
///   adler.update(*b);
/// }
///
/// assert_eq!(adler.digest(), 0x11E60398);
/// ```
impl Adler32 {
    pub fn new() -> Adler32 {
        Adler32 { a: 1, b: 0 }
    }
}

impl Digest for Adler32 {
    fn update(&mut self, b: u8) {
        self.a = self.a.wrapping_add(b as u16);
        self.b = self.b.wrapping_add(self.a);
    }

    fn digest(&self) -> u32 {
        ((self.b as u32) << 16) + self.a as u32
    }

    fn reset(&mut self) {
        self.a = 1;
        self.b = 0;
    }
}
