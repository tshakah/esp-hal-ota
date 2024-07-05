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

// NOTE: I need to use generics, because after adding esp-storage dependency to
// this project its not compiling LULE
pub struct Ota<S>
where
    S: ReadStorage + Storage,
{
    flash: S,

    last_crc: u32,
    ota_offset: Option<u32>,
    target_partition: Option<usize>,
    flash_size: u32,
    ota_remaining: u32,
}

// TODO: add OtaError enum

impl<S> Ota<S>
where
    S: ReadStorage + Storage,
{
    pub fn new(flash: S) -> Self {
        Ota {
            flash,

            last_crc: 0,
            ota_offset: None,
            target_partition: None,
            ota_remaining: 0,
            flash_size: 0,
        }
    }

    /// Sets ota_offset as next partitions offset
    pub fn with_next_partition_offset(self) -> Self {
        let next_part = Self::get_next_ota_partition();
        let ota_offset = next_part.map(|i| PARTITIONS[i].start);

        Ota {
            ota_offset,
            target_partition: next_part,
            ..self
        }
    }

    /// Sets the firmware flash size
    pub fn set_flash_size(&mut self, size: u32) {
        self.ota_remaining = size;
        self.flash_size = size;
    }

    /// Returns ota progress in f32 (0..1)
    pub fn get_ota_progress(&self) -> f32 {
        (self.flash_size - self.ota_remaining) as f32 / self.flash_size as f32
    }

    /// Writes next firmware chunk
    pub fn ota_write_chunk(&mut self, chunk: &[u8]) -> Result<bool, ()> {
        if self.flash_size == 0 {
            log::error!("[OTA] Cant write chunk without set_flash_size()");
            return Err(());
        }

        if self.ota_remaining == 0 {
            return Ok(true);
        }

        let ota_offset = self.ota_offset.as_mut().ok_or_else(|| ())?;
        let write_size = chunk.len() as u32;
        let write_size = write_size.min(self.ota_remaining) as usize;

        self.flash
            .write(*ota_offset, &chunk[..write_size])
            .map_err(|_| ())?;

        log::debug!(
            "[OTA] Wrote {} bytes to ota partition at 0x{:x}",
            write_size,
            ota_offset
        );

        self.last_crc = crc32::calc_crc32(&chunk[..write_size], self.last_crc);

        *ota_offset += write_size as u32;
        self.ota_remaining -= write_size as u32;
        Ok(self.ota_remaining == 0)
    }

    // TODO: crc checks or sth
    pub fn ota_flush(&mut self) -> Result<(), ()> {
        if let Some(target_partition) = self.target_partition {
            self.set_target_ota_boot_partition(target_partition);
        }

        log::info!("Calculated crc hash: {:?}", self.last_crc);

        Ok(())
    }

    /// Sets ota boot target partition
    pub fn set_target_ota_boot_partition(&mut self, target: usize) {
        let (seq1, seq2) = self.get_ota_boot_sequences();

        let mut target_seq = seq1.max(seq2);
        while helpers::seq_to_part(target_seq) != target || target_seq == 0 {
            target_seq += 1;
        }

        let flash = &mut self.flash;
        let target_crc = crc32::calc_crc32(&target_seq.to_le_bytes(), 0xFFFFFFFF);
        if seq1 > seq2 {
            let offset = OTADATA_OFFSET + (OTADATA_SIZE >> 1);

            _ = flash.write(offset, &target_seq.to_le_bytes());
            _ = flash.write(offset + 32 - 4, &target_crc.to_le_bytes());
        } else {
            _ = flash.write(OTADATA_OFFSET, &target_seq.to_le_bytes());
            _ = flash.write(OTADATA_OFFSET + 32 - 4, &target_crc.to_le_bytes());
        }
    }

    /// Returns current OTA boot sequences
    ///
    /// NOTE: if crc doesn't match, it returns 0 for that seq
    pub fn get_ota_boot_sequences(&mut self) -> (u32, u32) {
        let mut bytes = [0; 32];

        _ = self.flash.read(OTADATA_OFFSET, &mut bytes);
        let crc1 = u32::from_le_bytes(bytes[(32 - 4)..32].try_into().unwrap());
        let seq1 = helpers::seq_or_default(&bytes[..4], crc1, 0);

        _ = self
            .flash
            .read(OTADATA_OFFSET + (OTADATA_SIZE >> 1), &mut bytes);
        let crc2 = u32::from_le_bytes(bytes[(32 - 4)..32].try_into().unwrap());
        let seq2 = helpers::seq_or_default(&bytes[..4], crc2, 0);

        (seq1, seq2)
    }

    /// Returns currently booted partition index
    pub fn get_currently_booted_partition() -> Option<usize> {
        paddr::esp_get_current_running_partition()
    }

    /// BUG: this wont work if user has ota partitions not starting from ota0
    /// or if user skips some ota partitions: ota0, ota2, ota3...
    ///
    /// NOTE: This isn't reading from ota_boot_sequences, maybe in the future
    /// it will read from them to eliminate possibility of wrong PADDR result.
    /// (ESP-IDF has if's for PADDR-chain so it can fail somehow)
    pub fn get_next_ota_partition() -> Option<usize> {
        let curr_part = paddr::esp_get_current_running_partition();
        curr_part.map(|next_part| (next_part + 1) % PARTITIONS_COUNT)
    }
}
