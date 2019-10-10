use core::convert::TryInto;
use core::fmt::{Error, Write};


pub struct Uart
{
  base_address: usize,
}

impl Write for Uart
{
  fn write_str(&mut self, s: &str) -> Result<(), Error>
  {
    for c in s.bytes()
    {
      self.put(c);
    }
    Ok(())
  }
}

impl Uart
{
  pub fn new(base_address: usize) -> Self
  {
    Uart
    {
      base_address
    }
  }

  pub fn init(&mut self)
  {
    
    let ptr = self.base_address as *mut u8;

    unsafe
    {

      // Rust gets mad if you don't use snake case
      let ier = ptr.add(1) as *mut u8;
      let fcr = ptr.add(2) as *mut u8;
      let lcr = ptr.add(3) as *mut u8;

      // Divisor registers. DLL holds the lower 8 bits, and DLM holds the higher
      // 8 bits.
      let dll = ptr as *mut u8;
      let dlm = ptr.add(1) as *mut u8;

      // Set word length which are bits 0 and 1 of the LCR 
      lcr.write_volatile((1 << 0) | (1 << 1));
      // Enable the FIFO
      fcr.write_volatile(1 << 0);
      // Enable receiver buffer interupts
      ier.write_volatile(1 << 0);
      
      // Calculate the divisor
      let divisor: u16 = 592;
      let divisor_least = (divisor & 0xff).try_into().unwrap();
      let divisor_most = ((divisor >> 8) & 0xff).try_into().unwrap();

      // Set the DLAB
      lcr.write_volatile(1 << 7);

      // Put the divisor into DLL and DLM
      dll.write_volatile(divisor_least);
      dlm.write_volatile(divisor_most);

      // Clear the DLAB
      lcr.write_volatile(lcr.read_volatile() & !(1 << 7));

    }
  }

  pub fn put(&mut self, c:u8)
  {
    let ptr = self.base_address as *mut u8;
    unsafe
    {
      let thr = ptr.add(0) as *mut u8;
      thr.write_volatile(c);
    }
  }

  pub fn get(&mut self) -> Option<u8>
  {
    let ptr = self.base_address as *mut u8;
    unsafe
    {
      let rbr = ptr.add(0) as *mut u8;
      let lsr = ptr.add(5) as *mut u8;
      // If the LSR does not have data ready, ready None
      if lsr.read_volatile() & 1 == 0
      {
        None
      }
      else
      {
        Some(rbr.read_volatile())
      }
    }
  }
}

