# esp-hal-ota
OTA for esp-hal (no-std).

## Limitations
For now only works on esp32s3 (esp32c3 in the near future).

## Todo
- [ ] Fully working library
- [ ] Other esp32's (like esp32c3, esp32s2, etc..)
- [ ] Simple example

## Resources
- https://github.com/esp-rs/espflash (this led me to esp-idf-part)
- https://github.com/esp-rs/esp-idf-part
- https://github.com/espressif/esp-idf/blob/master/docs/en/api-reference/system/ota.rst (especially Python API)
- https://github.com/python/cpython/blob/main/Modules/binascii.c (internal_crc32)
- https://github.com/espressif/esp-idf/blob/master/components/app_update/esp_ota_ops.c#L552 (esp get current partition (paddr))
- https://github.com/bjoernQ/esp32c3-ota-experiment (i've only seen this after first experiments, so haven't looked at it yet)
