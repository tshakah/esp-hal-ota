const PARTITION_BYTES: &[u8] = include_bytes!("../part.bin");

fn main() {
    let chunks = PARTITION_BYTES.chunks(32);
    for chunk in chunks {
        let magic = &chunk[0..2];
        if magic != &[0xAA, 0x50] {
            continue;
        }

        let p_type = &chunk[2];
        let p_subtype = &chunk[3];
        let p_offset = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        let p_size = u32::from_le_bytes(chunk[8..12].try_into().unwrap());
        let p_name = core::str::from_utf8(&chunk[12..28]).unwrap();
        let p_flags = u32::from_le_bytes(chunk[28..32].try_into().unwrap());

        println!("{magic:?} {p_type} {p_subtype} {p_offset} {p_size} {p_name} {p_flags}");
    }
}
