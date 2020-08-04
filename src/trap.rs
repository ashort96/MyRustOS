// Adam Short
// 08/02/2020

use crate::cpu::TrapFrame;
use crate::{plic, uart};
use crate::syscall::do_syscall;
use crate::sched::schedule;

extern "C" {
	fn switch_to_user(frame: usize, mepc: usize, satp: usize) -> !;
}

#[no_mangle]
extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: *mut TrapFrame,
) -> usize {
    
    let is_async = {
        if cause >> 63 & 1 == 1 {
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
              let (frame, mepc, satp) = schedule();
              let mtimecmp = 0x0200_4000 as *mut u64;
              let mtime = 0x0200_bff8 as *const u64;
              mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
              switch_to_user(frame, mepc, satp);
            },
			      // Interrupt from PLIC
            11 => {
              if let Some(interrupt) = plic::next() {
                match interrupt {
                  // UART interrupt!
                  10 => {
                    let mut my_uart = uart::Uart::new(0x1000_0000);
                    if let Some(c) = my_uart.get() {
                      match c {
                        // Backspace
                        8 | 127 => {
                          print!("{} {}", 8 as char, 8 as char);
                        },
                        // \n or \r
                        10 | 13 => {
                          println!();
                        },
                        _ => {
                          print!("{}", c as char);
                        }
                      }
                    }
                  },
                  _ => {
                    println!("Non-UART external input: {}", interrupt);
                  }
                }
                plic::complete(interrupt);
              }
            },
            _ => { panic!("Unhandled async trap! CPU#{} -> {}\n", hart, cause_num); }
        }
    }
    else {
        match cause_num {
            2 => { 
              panic!("Illegal instruction! CPU#{} -> 0x{:08x}", hart, epc); 
              while true {}
            
            },
            8 => {
                return_pc = do_syscall(return_pc, frame);
            },
            9 => {
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc = do_syscall(return_pc, frame);
            },
            11 => { println!("E-call from Machine mode! CPU#{} -> 0x{:08x}", hart, epc); },
            12 => {
                println!("Instruction page fault! CPU#{} -> 0x{:08x}", hart, epc);
                while true {}
                return_pc += 4;
            },
            13 => {
                println!("Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                while true {}
                return_pc += 4;
            },
            15 => {
                println!("Store page fault! CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                while true {}
                return_pc += 4;
            },
            _ => { panic!("Unhandled sync trap! CPU#{} -> {}\n", hart, cause_num); }
        }
    };
    return_pc

}
