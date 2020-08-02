// Adam Short
// 08/02/2020

#![no_std]
#![feature(panic_info_message,asm)]


////////////////////////////////////////////////////////////////////////////////
// Rust Macros
////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! print
{
  ($($args:tt)+) => ({
    use core::fmt::Write;
    let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
  });
}

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

////////////////////////////////////////////////////////////////////////////////
// Language Structures / Functions
////////////////////////////////////////////////////////////////////////////////
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
extern "C"
fn abort() -> ! {
  loop {
    unsafe {
      asm!("wfi"::::"volatile");
    }
  }
}

#[no_mangle]
extern "C"
fn kmain() {

  let mut my_uart = uart::Uart::new(0x1000_0000);
  my_uart.init();

  println!("Short Operating System (SOS)");
  
}

pub mod uart;