// Adam Short
// 08/02/2020

#![no_std]
#![feature(panic_info_message,
           asm,
           allocator_api,
           alloc_error_handler,
           alloc_prelude,
           const_raw_ptr_to_usize_cast)]

// #[macro_use]
extern crate alloc;
// This is experimental and requires alloc_prelude as a feature
// use alloc::prelude::v1::*;

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////
#[macro_export]
macro_rules! print
{
  ($($args:tt)+) => ({
      use core::fmt::Write;
      let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
      });
}
#[macro_export]
macro_rules! println
{
  () => ({
       print!("\r\n")
       });
  ($fmt:expr) => ({
      print!(concat!($fmt, "\r\n"))
      });
  ($fmt:expr, $($args:tt)+) => ({
      print!(concat!($fmt, "\r\n"), $($args)+)
      });
}

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////
#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
  print!("Aborting: ");
  if let Some(p) = info.location() {
    println!(
             "line {}, file {}: {}",
             p.line(),
             p.file(),
             info.message().unwrap()
    );
  }
  else {
    println!("no information available.");
  }
  abort();
}
#[no_mangle]
extern "C" fn abort() -> ! {
  loop {
    unsafe {
      asm!("wfi"::::"volatile");
    }
  }
}

pub fn id_map_range(root: &mut page::Table,
                    start: usize,
                    end: usize,
                    bits: i64)
{
  let mut memaddr = start & !(page::PAGE_SIZE - 1);
  let num_kb_pages =
    (page::align_val(end, 12) - memaddr) / page::PAGE_SIZE;

  for _ in 0..num_kb_pages {
    page::map(root, memaddr, memaddr, bits, 0);
    memaddr += 1 << 12;
  }
}
// ///////////////////////////////////
// / ENTRY POINT
// ///////////////////////////////////
#[no_mangle]
extern "C" fn kinit() -> usize {
  uart::Uart::new(0x1000_0000).init();
  page::init();
  kmem::init();
  let ret = process::init();
  println!("Init process created at address 0x{:08x}", ret);
  plic::set_threshold(0);
  plic::enable(10);
  plic::set_priority(10, 1);
  println!("UART interrupts have been enabled and are awaiting your command.");
  println!("Getting ready for first process.");
  println!("Issuing the first context-switch timer.");
  unsafe {
    let mtimecmp = 0x0200_4000 as *mut u64;
    let mtime = 0x0200_bff8 as *const u64;
    mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
  }

  ret
}
#[no_mangle]
extern "C" fn kinit_hart(hartid: usize) {
  unsafe {

    cpu::mscratch_write((&mut cpu::KERNEL_TRAP_FRAME[hartid] as *mut cpu::TrapFrame) as usize);

    cpu::sscratch_write(cpu::mscratch_read());
    cpu::KERNEL_TRAP_FRAME[hartid].hartid = hartid;
  }
}

// ///////////////////////////////////
// / RUST MODULES
// ///////////////////////////////////

pub mod cpu;
pub mod kmem;
pub mod page;
pub mod plic;
pub mod process;
pub mod trap;
pub mod uart;