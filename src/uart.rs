// Adam Short
// 08/02/2020

use core::convert::TryInto;
use core::fmt::{Error, Write};

pub struct Uart {
    base_address: usize,
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart {

    ////////////////////////////////////////////////////////////////////////////
    // Create a new Uart
    ////////////////////////////////////////////////////////////////////////////
    pub fn new(base_address: usize) -> Self {
        Uart {
            base_address
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Initialize the UART.
    // - Set the word length to 8-bits by setting bits 1 and 0 of the LCR
    // - Enable FIFOs by setting bit 0 of the FCR
    // - Enable receiver interrupts by setting bit 0 of the IER
    // - Write the divisor latch (least and most) and let the word length 
    //   to 8-bits
    ////////////////////////////////////////////////////////////////////////////
    pub fn init(&mut self) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            let lcr = 0b11;
            ptr.add(3).write_volatile(lcr);
            ptr.add(2).write_volatile(0b01);
            ptr.add(1).write_volatile(0b01);

            let divisor: u16 = 592;
            let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
            let divisor_most: u8 = (divisor >> 8).try_into().unwrap();

            ptr.add(3).write_volatile((1 << 7) | lcr);
            ptr.add(0).write_volatile(divisor_least);
            ptr.add(1).write_volatile(divisor_most);

            ptr.add(3).write_volatile(lcr);
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Read from the UART
    // If the DR (data ready) bit of the LCR is toggled, read the data
    ////////////////////////////////////////////////////////////////////////////
    pub fn get(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                None
            }
            else {
                Some(ptr.add(0).read_volatile())
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Write to the UART
    ////////////////////////////////////////////////////////////////////////////
    pub fn put(&mut self, c: u8) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }
    
}

