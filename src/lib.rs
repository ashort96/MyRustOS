#![no_std]
#![feature(panic_info_message,asm)]


// Rust MACROS

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
    print!("\r\n");
  });
  
  ($fmt:expr) => ({
    print!(concat!($fmt, "\r\n"))
  });

  ($fmt:expr, $($args:tt)+) => ({
    print!(concat!($fmt, "\r\n"), $($args)+)
  });
}

// Language Structures / Functions

#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> !
{
  print!("Aborting: ");
  if let Some(p) = info.location()
  {
    println!(
      "line{}, file{}: {}",
      p.line(),
      p.file(),
      info.message().unwrap()
    );
  }
  else
  {
    println!("no information available.");
  }
  abort();
}

#[no_mangle]
extern "C"
fn abort() -> !
{
  loop
  {
    unsafe
    {
      asm!("wfi"::::"volatile");
    }
  }
}

// Entry point

#[no_mangle]
extern "C"
fn kmain()
{
  let mut my_uart = uart::Uart::new(0x1000_0000);
  my_uart.init();

  println!("This is my operating system!");
  loop
  {
    if let Some(c) = my_uart.get()
    {
      match c
      {
        // Backspace
        8 | 127 => {
          print!("{}{}{}", 8 as char, ' ', 8 as char);
        },
        // Newline or carriage-return (Enter)
        10 | 13 => {
          println!();
        },
        // ANSI escape sequences
        0x1b => {
          if let Some(next_byte) = my_uart.get()
          {
            if next_byte == 91
            {
              if let Some(b) = my_uart.get()
              {
                match b as char
                {
                  'A' => {
                    println!("Up arrow!");
                  },
                  'B' => {
                    println!("Down arrow!");
                  },
                  'C' => {
                    println!("Right arrow!");
                  },
                  'D' => {
                    println!("Left arrow!");
                  },
                  _ => {
                    println!("That is something else...");
                  }
                }
              }
            }
          }
        },
        _ => {
          print!("{}", c as char);
        }
      }
    }
  }
}

pub mod uart;
