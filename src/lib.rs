#![no_std]

use embedded_storage::{ReadStorage, Storage};

pub mod crc32;
pub mod helpers;
pub mod paddr;

// TOOD: make macros for generating this from partitions.csv, delete them from lib
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

impl<S> Ota<S> where S: ReadStorage + Storage {
    pub fn dsa(&mut self) {
        //self.flash.write(offset, bytes)
    }
}
