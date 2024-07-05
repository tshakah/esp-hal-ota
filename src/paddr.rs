// this is for esp32s3, TODO: implement for other esp32's
const SOC_MMU_VADDR_MASK: u32 = 0x1FFFFFF;
//const MMU_PAGE_64KB: u32 = 0x10000;
const DR_REG_MMU_TABLE: u32 = 0x600c5000;
const SOC_MMU_VALID_VAL_MASK: u32 = 0x3fff;

fn mmu_ll_get_entry_id(_mmu_id: u32, vaddr: u32) -> u32 {
    (vaddr & SOC_MMU_VADDR_MASK) >> 16
}

fn mmu_ll_entry_id_to_paddr_base(_mmu_id: u32, entry_id: u32) -> u32 {
    let ptr = (DR_REG_MMU_TABLE + entry_id * 4) as *const u32;
    unsafe { ((*ptr) & SOC_MMU_VALID_VAL_MASK) << 16 }
}

pub fn esp_get_current_running_partition(partitions: &[(u32, u32)]) -> Option<usize> {
    let ptr = esp_get_current_running_partition as *const () as *const u32;
    let entry_id = mmu_ll_get_entry_id(0, ptr as u32);

    let shift_code = 16; // bc MMU_PAGE_64K
    let mmu_hal_pages_to_bytes = 1 << shift_code;

    let offset = (ptr as u32) % mmu_hal_pages_to_bytes;

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
