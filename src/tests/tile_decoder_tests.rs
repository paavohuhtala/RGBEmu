use rendering::*;

#[test]
fn check_copyright_symbol() {
    let copyright_symbol = [
        0x3C, 0x00, 0x42, 0x00, 0xB9, 0x00, 0xA5, 0x00, 0xB9, 0x00, 0xA5, 0x00, 0x42, 0x00, 0x3C,
        0x00
    ];

    let mut decompressed_buffer = [0u8; 64];
    let copyright_tile_data = TileData(copyright_symbol);

    copyright_tile_data.unpack_to(&mut decompressed_buffer);

    println!("{:?}", &decompressed_buffer[0..8]);
    println!("{:?}", &decompressed_buffer[8..16]);
    println!("{:?}", &decompressed_buffer[16..24]);
    println!("{:?}", &decompressed_buffer[24..32]);
    println!("{:?}", &decompressed_buffer[32..40]);
    println!("{:?}", &decompressed_buffer[40..48]);
    println!("{:?}", &decompressed_buffer[48..56]);
    println!("{:?}", &decompressed_buffer[56..64]);
}
