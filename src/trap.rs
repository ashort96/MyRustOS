// Adam Short
// 08/02/2020

use crate::cpu::TrapFrame;

#[no_mangle]
extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: &mut TrapFrame,
) -> usize {
    
    let is_async = {
        if cause >> 63 & 0x1 == 1 {
            true
        }
         else {
             false
         }
    };

    let cause_num = cause & 0xfff;
    let mut return_pc = epc;

    if is_async {
        match cause_num {
            3 => { println!("Machine software interrupt! CPU#{}", hart); },
            // Machine timer
            7 => unsafe {
                let mtimecmp = 0x0200_4000 as *mut u64;
                let mtime = 0x0200_bff8 as *const u64;
                mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
            },
            11 => { println!("Machine external interrupt! CPU#{}", hart); },
            _ => { panic!("Unhandled async trap! CPU#{} -> {}\n", hart, cause_num); }
        }
    }
    else {
        match cause_num {
            2 => { panic!("Illegal instruction! CPU#{} -> 0x{:08x}", hart, epc); },
            8 => {
                println!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            },
            9 => {
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            },
            11 => { println!("Instruction page fault! CPU#{} -> 0x{:08x}", hart, epc); },
            12 => {
                println!("Load page fault! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            },
            13 => {
                println!("Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            },
            15 => {
                println!("Store page fault! CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            },
            _ => { panic!("Unhandled sync trap! CPU#{} -> {}\n", hart, cause_num); }
        }
    };
    return_pc

}