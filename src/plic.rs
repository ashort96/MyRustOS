// Adam Short
// 08/03/2020

const PLIC_PRIORITY: usize = 0x0c00_0000;
const PLIC_PENDING: usize = 0x0c00_1000;
const PLIC_INT_ENABLE: usize = 0x0c00_2000;
const PLIC_THRESHOLD: usize = 0x0c20_0000;
const PLIC_CLAIM: usize = 0x0c20_0004;

pub fn next() -> Option<u32> {
    let claim_reg = PLIC_CLAIM as *const u32;
    let claim_no;
    unsafe {
        claim_no = claim_reg.read_volatile();
    }
    if claim_no == 0 {
        None
    }
    else {
        Some(claim_no)
    }
}


pub fn complete(id: u32) {
    let complete_reg = PLIC_CLAIM as *mut u32;
    unsafe { complete_reg.write_volatile(id); }
}

pub fn set_threshold(tsh: u8) {
    let actual_tsh = tsh & 7;
    let tsh_reg = PLIC_THRESHOLD as *mut u32;
    unsafe { tsh_reg.write_volatile(actual_tsh as u32); }
}

pub fn is_pending(id: u32) -> bool {
    let pend = PLIC_PENDING as *const u32;
    let actual_id = 1 << id;
    let pend_ids;
    unsafe { pend_ids = pend.read_volatile(); }
    actual_id & pend_ids != 0
}

pub fn enable(id: u32) {
    let enables = PLIC_INT_ENABLE as *mut u32;
    let actual_id = 1 << id;
    unsafe { enables.write_volatile(enables.read_volatile() | actual_id); }
}

pub fn set_priority(id: u32, prio: u8) {
    let actual_prio = prio as u32 & 7;
    let prio_reg = PLIC_PRIORITY as *mut u32;
    unsafe { prio_reg.add(id as usize).write_volatile(actual_prio); }
}