# esp-hal-ota
OTA for esp-hal (no-std).

[![crates.io](https://img.shields.io/crates/v/mdns.svg)](https://crates.io/crates/esp-hal-ota)
[![MIT license](https://img.shields.io/github/license/mashape/apistatus.svg)]()

## Limitations
For now only works on esp32s3 (esp32c3 in the near future).

## Features
- Obviously OTA updates
- Dynamic partitions reading (so no macros, no reading from partitions.csv) - fully automatic
- Checking currently booted partition (using some pointer magic from ESP-IDF)
- CRC32 verification

## Example
To see real-world example look at `./examples` and `./simple-ota-server` dirs.

```rust
let flash_size = 1234; // get it from OTA server
let target_crc = 65436743; // get it from OTA server

let mut ota = Ota::new(FlashStorage::new()).unwrap();
ota.ota_begin(flash_size, target_crc);

let mut buf = [0; 4096];
loop {
    let n = read_next_chunk(&mut buf);
    if n == 0 {
        break;
    }

    let res = ota.ota_write_chunk(&buf[..n]);
    if res == Ok(true) { // end of flash
        if ota.ota_flush(true).is_ok() { // true if you want to verify crc reading flash
            esp_hal::reset::software_reset();
        }
    }

    let progress = (ota.get_ota_progress() * 100) as u8;
    log::info!("progress: {}%", progress);
}
```

## Todo
- [x] Fully working library
- [x] Simple example
- [x] Better errors
- [ ] Other esp32's (like esp32c3, esp32s2, etc..)
- [ ] Rollbacks

## Resources
- https://github.com/esp-rs/espflash (this led me to esp-idf-part)
- https://github.com/esp-rs/esp-idf-part
- https://github.com/espressif/esp-idf/blob/master/docs/en/api-reference/system/ota.rst (especially Python API)
- https://github.com/python/cpython/blob/main/Modules/binascii.c (internal_crc32)
- https://github.com/espressif/esp-idf/blob/master/components/app_update/esp_ota_ops.c#L552 (esp get current partition (paddr))
- https://github.com/bjoernQ/esp32c3-ota-experiment (i've only seen this after first experiments, so haven't looked at it yet)
