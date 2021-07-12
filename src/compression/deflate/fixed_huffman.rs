const INVALID_HUFFMAN_CODE: u16 = u16::MAX;

const fn make_fixed_huffman_table() -> [u16; 512] {
    let mut n: usize = 0;
    let mut table: [u16; 512] = [INVALID_HUFFMAN_CODE; 512];

    while n < 0b001100000 {
        table[n] = (256 + (n >> 2)) as u16;
        n += 1;
    }
    while n < 0b110000000 {
        table[n] = (n >> 1) as u16 - 0b00110000;
        n += 1;
    }
    while n < 0b110010000 {
        table[n] = (n >> 1) as u16 - 0b11000000 + 280;
        n += 1;
    }
    while n <= 0b111111111 {
        table[n] = (n >> 1) as u16 - 0b11001000 + 144;
        n += 1;
    }
    table
}

const FIXED_HUFFMAN_TABLE: [u16; 512] = make_fixed_huffman_table();

fn fixed_huffman_value(compressed_symbol_9_bits: u64) -> u16 {
    assert!(compressed_symbol_9_bits < 512);
    FIXED_HUFFMAN_TABLE[compressed_symbol_9_bits as usize]
}
