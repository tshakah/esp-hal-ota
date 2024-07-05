# esp-hal-ota
OTA for esp-hal (no-std).

## Limitations
For now only works on esp32s3 (esp32c3 in the near future).

## Example
This example uses embassy-net for TcpSocket, as you can see ota doesn't use 
async so you can easily use it with smoltcp.

```rust
let mut ota_buff = [0; 4096];
socket
    .read(&mut ota_buff[..4])
    .await
    .expect("Cannot read firmware size!");

let flash_size = u32::from_le_bytes(ota_buff[..4].try_into().unwrap());
log::info!("flash_size: {flash_size}");

let mut ota = Ota::new(FlashStorage::new()).with_next_partition_offset();
ota.set_flash_size(flash_size);

let mut bytes_read = 0;
loop {
    let res = socket.read(&mut ota_buff).await;
    if let Ok(n) = res {
        bytes_read += n;
        if n == 0 {
            break;
        }

        let res = ota.ota_write_chunk(&ota_buff[..n]);
        if bytes_read % 4096 == 0 {
            _ = socket.write(&[0]).await;
        }

        if res == Ok(true) {
            _ = ota.ota_flush();
            break;
        }
    }

    Timer::after_millis(10).await;
    log::info!("Progress: {}%", (ota.get_ota_progress() * 100.0) as u8);
}
```

## Todo
- [ ] Fully working library
- [ ] Partitions macro (to generate PARTITIONS const at compile time)
- [ ] Other esp32's (like esp32c3, esp32s2, etc..)
- [ ] Rollbacks
- [x] Simple example

## Resources
- https://github.com/esp-rs/espflash (this led me to esp-idf-part)
- https://github.com/esp-rs/esp-idf-part
- https://github.com/espressif/esp-idf/blob/master/docs/en/api-reference/system/ota.rst (especially Python API)
- https://github.com/python/cpython/blob/main/Modules/binascii.c (internal_crc32)
- https://github.com/espressif/esp-idf/blob/master/components/app_update/esp_ota_ops.c#L552 (esp get current partition (paddr))
- https://github.com/bjoernQ/esp32c3-ota-experiment (i've only seen this after first experiments, so haven't looked at it yet)
