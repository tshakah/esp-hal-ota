use crate::{crc32, PARTITIONS_COUNT};

#[inline(always)]
/// Helper funcion!
/// Change seq to partition number
pub const fn seq_to_part(seq: u32) -> usize {
    (PARTITIONS_COUNT - 1) - (seq as usize % PARTITIONS_COUNT)
}

#[inline(always)]
/// Helper funcion!
/// If crc of seq is correct it returns seq, otherwise default value is returned
pub fn seq_or_default(seq: &[u8], crc: u32, default: u32) -> u32 {
    let crc_calc = crc32::calc_crc32(&seq, 0xFFFFFFFF);
    if crc == crc_calc {
        return u32::from_le_bytes(seq.try_into().expect("Wrong size?"));
    }

    default
}
