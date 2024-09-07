// this is for esp32s3, TODO: implement for other esp32's

struct ExtMemDefs {}
#[cfg(all(
    //not(feature = "esp32"),
    //not(feature = "esp32c2"),
    not(feature = "esp32c3"),
    //not(feature = "esp32c6"),
    //not(feature = "esp32h2"),
    not(feature = "esp32s2"),
    not(feature = "esp32s3")
))]
impl ExtMemDefs {
    const SOC_MMU_VADDR_MASK: u32 = 0;
    const DR_REG_MMU_TABLE: u32 = 0;
    const SOC_MMU_VALID_VAL_MASK: u32 = 0;
    const MMU_PAGE: u32 = 0;
}

#[cfg(feature = "esp32c3")]
impl ExtMemDefs {
    const SOC_MMU_VADDR_MASK: u32 = 0x7FFFFF;
    const DR_REG_MMU_TABLE: u32 = 0x600c5000;
    const SOC_MMU_VALID_VAL_MASK: u32 = 0xff;
    const MMU_PAGE: u32 = 0x10000;
}

#[cfg(feature = "esp32s2")]
impl ExtMemDefs {
    const SOC_MMU_VADDR_MASK: u32 = 0x3FFFFF;
    const DR_REG_MMU_TABLE: u32 = 0x61801000;
    const SOC_MMU_VALID_VAL_MASK: u32 = 0x3fff;
    const MMU_PAGE: u32 = 0x10000;
}

#[cfg(feature = "esp32s3")]
impl ExtMemDefs {
    const SOC_MMU_VADDR_MASK: u32 = 0x1FFFFFF;
    const DR_REG_MMU_TABLE: u32 = 0x600c5000;
    const SOC_MMU_VALID_VAL_MASK: u32 = 0x3fff;
    const MMU_PAGE: u32 = 0x10000;
}

fn mmu_ll_get_entry_id(_mmu_id: u32, vaddr: u32) -> u32 {
    (vaddr & ExtMemDefs::SOC_MMU_VADDR_MASK) >> 16
}

fn mmu_ll_entry_id_to_paddr_base(_mmu_id: u32, entry_id: u32) -> u32 {
    let ptr = (ExtMemDefs::DR_REG_MMU_TABLE + entry_id * 4) as *const u32;
    unsafe { ((*ptr) & ExtMemDefs::SOC_MMU_VALID_VAL_MASK) << 16 }
}

fn mmu_hal_pages_to_bytes(_mmu_id: u32, page_num: u32) -> u32 {
    let shift_code = match ExtMemDefs::MMU_PAGE {
        0x10000 => 16,
        0x8000 => 15,
        0x4000 => 14,
        _ => panic!("WRONG MMU_PAGE SIZE! {:X?}", ExtMemDefs::MMU_PAGE),
    };

    page_num << shift_code
}

pub fn esp_get_current_running_partition(partitions: &[(u32, u32)]) -> Option<usize> {
    let ptr = esp_get_current_running_partition as *const () as *const u32;
    let entry_id = mmu_ll_get_entry_id(0, ptr as u32);

    let page_size_in_bytes = mmu_hal_pages_to_bytes(0, 1);
    let offset = (ptr as u32) % page_size_in_bytes;

    let paddr_base = mmu_ll_entry_id_to_paddr_base(0, entry_id);
    let paddr = paddr_base | offset;

    for i in 0..partitions.len() {
        let part = partitions[i];

        if paddr >= part.0 && paddr < part.0 + part.1 {
            return Some(i);
        }
    }

    None
}
