use std::io;
use std::io::Read;

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
/// use vekotin::digest::Crc32;
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
    pub fn update(&mut self, b: u8) -> &mut Self {
        self.crc = CRC_TABLE[((self.crc ^ b as u32) & 0xff) as usize] ^ (self.crc >> 8);
        self
    }

    pub fn digest(&self) -> u32 {
        self.crc ^ 0xffffffff
    }
}

/// # Examples
///
/// ```rust
/// use std::io::Read;
/// use vekotin::digest::CrcReader;
///
/// let input = vec![0x49 as u8, 0x48, 0x44, 0x52, 0x00, 0x00, 0x03, 0x20, 0x00, 0x00, 0x02, 0x58,
/// 0x08, 0x06, 0x00, 0x00, 0x00];
/// let mut reader = CrcReader::new(input.as_slice());
///
/// let mut output = vec![0 as u8; 17];
///
/// reader.read(&mut output).unwrap();
///
/// assert_eq!(input, output);
/// assert_eq!(reader.digest(), 2591457904);
/// ```
pub struct CrcReader<R> {
    inner: R,
    crc: Crc32,
}

impl<R: Read> CrcReader<R> {
    pub fn new(inner: R) -> CrcReader<R> {
        CrcReader {
            inner,
            crc: Crc32::new(),
        }
    }

    pub fn digest(&self) -> u32 {
        self.crc.digest()
    }

    pub fn reset_crc(&mut self) -> &mut Self {
        self.crc = Crc32::new();
        self
    }
}

impl<R> CrcReader<R> {
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

impl<R: Read> Read for CrcReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n_read = self.inner.read(buf)?;
        for b in buf.iter().take(n_read) {
            self.crc.update(*b);
        }
        Ok(n_read)
    }
}
