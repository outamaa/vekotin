const CONSULT_9_BIT_TABLE: u16 = u16::MAX;

const fn make_static_huffman_table() -> [u16; 256] {
    let mut n: usize = 0;
    let mut table: [u16; 256] = [0; 256];

    while n < 0b00110000 {
        table[n] = (256 + (n >> 1)) as u16;
        n += 1;
    }
    while n < 0b11000000 {
        table[n] = n as u16 - 0b00110000;
        n += 1;
    }
    while n < 0b11001000 {
        table[n] = n as u16 - 0b11000000 + 280;
        n += 1;
    }
    while n < 0b11111111 {
        table[n] = CONSULT_9_BIT_TABLE;
        n += 1;
    }
    table
}

const STATIC_HUFFMAN_TABLE: [u16; 256] = make_static_huffman_table();
