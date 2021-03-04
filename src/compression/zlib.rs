#[derive(PartialEq, Debug)]
pub enum CompressionMethod {
    Deflate {
        window_size: u32,
    },
    Unknown,
}

impl From<u8> for CompressionMethod {
    fn from(b: u8) -> Self {
        use CompressionMethod::*;
        let cm = b & 0b00001111;
        let cinfo = b >> 4;

        if cm == 8 {
            // TODO: see http://optipng.sourceforge.net/pngtech/zlib-spec-correction.html
            let window_size = u32::pow(2, cinfo as u32 + 8);
            Deflate { window_size }
        } else {
            Unknown
        }
    }
}