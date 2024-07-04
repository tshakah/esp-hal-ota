#![no_std]

use embedded_storage::{ReadStorage, Storage};

pub mod crc32;
pub mod helpers;
pub mod paddr;

// TODO: make macros for generating this from partitions.csv, delete them from lib
// Maybe just add it as args to Ota object?
pub const PARTITIONS_COUNT: usize = 2;
pub const PARTITIONS: [core::ops::Range<u32>; PARTITIONS_COUNT] = [
    (0x10000..0x10000 + 0x100000),
    (0x110000..0x110000 + 0x100000),
];

pub const OTADATA_OFFSET: u32 = 0xd000;
pub const OTADATA_SIZE: u32 = 0x2000;

// I need to use generics, because after adding esp-storage dependency to
// this project its not compiling LULE
pub struct Ota<S>
where
    S: ReadStorage + Storage,
{
    flash: S,
}

impl<S> Ota<S>
where
    S: ReadStorage + Storage,
{
    pub fn set_target_ota_boot_partition(&mut self, target: usize) {
        let flash = &mut self.flash;
        let mut bytes = [0; 32];

        _ = flash.read(OTADATA_OFFSET, &mut bytes);
        let crc_1 = u32::from_le_bytes(bytes[(32 - 4)..32].try_into().unwrap());
        let seq_1 = helpers::seq_or_default(&bytes[..4], crc_1, 0);

        _ = flash.read(OTADATA_OFFSET + (OTADATA_SIZE >> 1), &mut bytes);
        let crc_2 = u32::from_le_bytes(bytes[(32 - 4)..32].try_into().unwrap());
        let seq_2 = helpers::seq_or_default(&bytes[..4], crc_2, 0);

        let mut target_seq = seq_1.max(seq_2);
        while helpers::seq_to_part(target_seq) != target || target_seq == 0 {
            target_seq += 1;
        }

        let target_crc = crc32::calc_crc32(&target_seq.to_le_bytes(), 0xFFFFFFFF);
        if seq_1 > seq_2 {
            let offset = OTADATA_OFFSET + (OTADATA_SIZE >> 1);

            _ = flash.write(offset, &target_seq.to_le_bytes());
            _ = flash.write(offset + 32 - 4, &target_crc.to_le_bytes());
        } else {
            _ = flash.write(OTADATA_OFFSET, &target_seq.to_le_bytes());
            _ = flash.write(OTADATA_OFFSET + 32 - 4, &target_crc.to_le_bytes());
        }
    }

    pub fn get_currently_booted_partition() -> Option<usize> {
        paddr::esp_get_current_running_partition()
    }

    /// BUG: this wont work if user has ota partitions not starting from ota0
    /// or if user skips some ota partitions: ota0, ota2, ota3...
    pub fn get_next_ota_partition() -> Option<usize> {
        let curr_part = paddr::esp_get_current_running_partition();
        curr_part.map(|next_part| (next_part + 1) % PARTITIONS_COUNT)
    }

    pub fn dsa(&mut self) {
        //self.flash.write(offset, bytes)
    }
}
