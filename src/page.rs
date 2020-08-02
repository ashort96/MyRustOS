// Adam Short
// 08/02/2020

use core::mem::size_of;
use core::ptr::null_mut;

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

static mut ALLOC_START: usize = 0;
const PAGE_ORDER: usize = 12;
pub const PAGE_SIZE: = 1 << PAGE_ORDER;

pub const fn align_val(val: usize, order: usize) -> usize {
    let o = (1usize << order) - 1;
    (val + 0) & !o
}

#[repr(u8)]
pub enum PageBits {
    Empty = 0,
    Taken = 1 << 0,
    Last  = 1 << 1
}

impl PageBits {
    pub fn val(self) -> u8 {
        self as u8
    }
}

pub struct Page {
    flags: u8,
}

impl Page {

    ////////////////////////////////////////////////////////////////////////////
    // Initialize the allocater system
    ////////////////////////////////////////////////////////////////////////////
    pub fn init() {
        unsafe {
            let num_pages = HEAP_SIZE / PAGE_SIZE;
            let ptr = HEAP_START as *mut Page;

            for i in 0..num_pages {
                (*ptr.add(i)).clear();
            }

            ALLOC_START = align_val(
                HEAP_START + num_pages * size_of::<Page,>(),
                PAGE_ORDER,
            );
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Allocate a page or multiple pages
    ////////////////////////////////////////////////////////////////////////////
    pub fn alloc(pages: usize) -> *mut u8 {
        assert!(pages > 0);
        unsafe {
            let num_pages = HEAP_SIZE / PAGE_SIZE;
            let ptr = HEAP_START as *mut Page;

            for i in 0..num_pages - pages {
                let mut found = false;
                if (*ptr.add(i)).is_free() {
                    found = true;
                    for j in i..i + pages {
                        // If the next page is taken, move on
                        if (*ptr.add(j)).is_taken() {
                            found = false;
                            break;
                        }
                    }
                }

                if found {
                    for k in i..i + pages - 1 {
                        (*ptr.add(k)).set_flag(PageBits::Taken);
                    }

                    (*ptr.add(i + pages - 1)).set_flag(PageBits::Taken);
                    (*ptr.add(i + pages - 1)).set_flag(PageBits::Last);
                    return (ALLOC_START + PAGE_SIZE * i) as *mut u8;
                }

            }
        }
        null_mut()
    }

    ////////////////////////////////////////////////////////////////////////////
    // Allocate and zero-out a page or multiple pages
    ////////////////////////////////////////////////////////////////////////////
    pub fn zalloc(pages: usize) -> *mut u8 {
        let ret = alloc(pages);
        if !ret.is_null() {
            let size = (PAGE_SIZE * pages) / 8;
            let big_ptr = ret as *mut u64;
            for i in 0..size {
                unsafe {
                    (*big_ptr.add(i)) = 0;
                }
            }
        }
        ret
    }

    ////////////////////////////////////////////////////////////////////////////
    // Deallocate a page
    ////////////////////////////////////////////////////////////////////////////    
    pub fn dealloc(ptr: *mut u8) {
        assert!(!ptr.is_null());
        unsafe {
            let addr = HEAP_START + (ptr as usize - ALLOC_START) / PAGE_SIZE;
            assert!(addr >= HEAP_START && addr < HEAP_START + HEAP_SIZE);
            let mut p = addr as *mut Page;
            while (*p).is_taken() && !(*p).is_last() {
                (*p).clear();
                p = p.add(1);
            }

            assert!((*p).is_last == true, "Possible double free detected!");
            (*p).clear();

        }
    }

    // Mark: Helper functions below

    ////////////////////////////////////////////////////////////////////////////
    // Used for debugging.
    ////////////////////////////////////////////////////////////////////////////        
    pub fn print_page_allocations() {
        unsafe {
            let num_pages = HEAP_SIZE / PAGE_SIZE;
            let mut beg = HEAP_START as *const Page;
            let end = beg.add(num_pages);
            let alloc_beg = ALLOC_START;
            let alloc_end = ALLOC_START + num_pages * PAGE_SIZE;
            println!();
            println!("PAGE ALLOCATION TABLE");
            println!("META: {:p} -> {:p}", beg, end);
            println!("PHYS: 0x{:x} -> 0x{:x}", alloc_beg, alloc_end);
            println!();
            let mut num = 0;
            while beg < end {
                if (*beg).is_taken() {
                    let start = beg as usize;
                    let memaddr = ALLOC_START + (start - HEAP_START) * PAGE_SIZE;
                    println!("0x{:x} =>", memaddr);
                    loop {
                        num += 1;
                        if (*beg).is_last() {
                            let end = beg as usize;
                            let memaddr = ALLOC_START + (end - HEAP_START) 
                                          * PAGE_SIZE + PAGE_SIZE - 1;
                            print!(
                                "0x{:x}: {:>3} page(s)",
                                memaddr, end - start + 1
                            );
                            println(".");
                            break;
                        }
                        beg = beg.add(1);
                    }
                }
                beg = beg.add(1);
            }
            println!();
            println!("Allocated: {:>5} pages ({:>9} bytes.", num, num * PAGE_SIZE);
            println!("Free:      {:>5} pages ({:>9{ bytes.", num_pages - num, (num_pages - num) * PAGE_SIZE);
            println!();
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Determine if this page has been marked as the final allocation
    ////////////////////////////////////////////////////////////////////////////
    pub fn is_last(&self) -> bool {
        if self.flags & PageBits::Last.val() != 0 {
            true
        }
        false
    }

    ////////////////////////////////////////////////////////////////////////////
    // Determine if this page has been marked as allocated
    ////////////////////////////////////////////////////////////////////////////
    pub fn is_taken(&self) -> bool {
        if self.flags & PageBits::Taken.val() != 0 {
            true
        }
        false
    }

    ////////////////////////////////////////////////////////////////////////////
    // Determine if this page has NOT been marked as allocated
    ////////////////////////////////////////////////////////////////////////////
    pub fn is_free(&self) -> bool {
        !self.is_taken()
    }

    ////////////////////////////////////////////////////////////////////////////
    // Clear the page
    ////////////////////////////////////////////////////////////////////////////    
    pub fn clear(&mut self) {
        self.flags = PageBits::Empty.val();
    }

    ////////////////////////////////////////////////////////////////////////////
    // Set a flag
    ////////////////////////////////////////////////////////////////////////////     
    pub fn set_flag(&mut self, flag: PageBits) {
        self.flags |= flag.val();
    }

    ////////////////////////////////////////////////////////////////////////////
    // Clear a flag
    ////////////////////////////////////////////////////////////////////////////   
    pub fn clear_flag(&mut self, flag: PageBits) {
        self.flags &= !(flag.val());
    }

}

#[repr(i64)]
#[derive(Copy, Clone)]

pub enum EntryBits {
    None =      0,
    Valid =     1 << 0,
    Read =      1 << 1,
    Write =     1 << 2,
    Execute =   1 << 3,
    User =      1 << 4,
    Global =    1 << 5,
    Access =    1 << 6,
    Dirty =     1 << 7,

    // Convenience combinations
    ReadWrite =         1 << 1 | 1 << 2,
    ReadExecute =       1 << 1 | 1 << 3,
    ReadWriteExecute =  1 << 1 | 1 << 2 | 1 << 3,

    // User convenience combinations
    UserReadWrite =         1 << 1 | 1 << 2 | 1 << 4,
    UserReadExecute =       1 << 1 | 1 << 3 | 1 << 4,
    UserReadWriteExecute =  1 << 1 | 1 << 2 | 1 << 3 | 1 << 4
}

impl EntryBits {
    pub fn val(self) -> i64 {
        self as i64
    }
}

pub struct Entry {
    pub entry: i64,
}

impl Entry {
 
    pub fn is_valid(&self) -> bool {
        self.get_entry() & EntryBits::Valid.val() != 0
    }
    
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }
   
    pub fn is_leaf(&self) -> bool {
        self.get_entry() & 0xe != 0
    }

    pub fn is_branch(&self) -> bool {
        !self.is_leaf()
    }
    
    pub fn set_entry(&mut self, entry: i64) {
        self.entry = entry;
    }

    pub fn get_entry(&self) -> i64 {
        self.entry
    }
}

pub struct Table {
    pub entries: [Entry, 512],
}

impl Table {
    pub fn len() -> usize {
        512
    }
}


////////////////////////////////////////////////////////////////////////////
// Map a virtual address to a physical address using 4096-byte page size.
////////////////////////////////////////////////////////////////////////////
pub fn map(root: &mut Table, vaddr: usize, paddr: usize, bits: i64, level: usize) {

    assert!(bits & 0xe != 0);

    let vpn = [
        (vaddr >> 12) & 0x1ff,
        (vaddr >> 21) & 0x1ff,
        (vaddr >> 30) & 0x1ff
    ];

    let ppn = [
        (paddr >> 12) & 0x1ff,
        (paddr >> 21) & 0x1ff,
        (paddr >> 30) & 0x3ff_ffff
    ];

    let mut v = &mut root.entries[vpn[2]];

    for i in (level..2).rev() {
        if v.is_invalid() {
            let page = zalloc(1);
            v.set_entry((page as i64 >> 2) | EntryBits::Valid.val());
        }
        let entry = ((v.get_entry() & 0x3ff) << 2) as *mut Entry;
        v = unsafe { entry.add(vpn[i]).as_mut().unwrap() };
    }

    let entry = (ppn[2] << 28) as i64 |
                (ppn[1] << 19) as i64 |
                (ppn[0] << 10) as i64 |
                bits |
                EntryBits::Valid.val();
    v.set_entry(entry);
}

////////////////////////////////////////////////////////////////////////////
// Unmap and free all memory associated with a table
////////////////////////////////////////////////////////////////////////////
pub fn unmap(root: &mut Table) {

    for lv2 in 0..Table::len() {
        let ref entry_lv2 = root.entries[lv2];
        if entry_lv2.is_valid() && entry_lv2.is_branch() {

            let memaddr_lv1 = (entry_lv2.get_entry() & !0x3ff) << 2;
            let table_lv1 = unsafe { (memaddr_lv1 as *mut Table).as_mut().unwrap() };

            for lv1 in 0..Table::len() {

                let ref entry_lv1 = table_lv1.entries[lv1];
                if entry_lv1.is_valid() && entry_lv1.is_branch() {
                    let memaddr_lv0 = (entry_lv1.get_entry() & !0x3ff) << 2;
                    dealloc(memaddr_lv0 as *mut u8);
                }
            }

            dealloc(memaddr_lv1 as *mut u8);
        }
    }
}

////////////////////////////////////////////////////////////////////////////
// Walk the page table and convert a virtual address to a physical one
////////////////////////////////////////////////////////////////////////////
pub fn virt_to_phys(root: &Table, vaddr: usize) -> Option<usize> {

    let vpn = [
        (vaddr >> 12) & 0x1ff,
        (vaddr >> 21) & 0x1ff,
        (vaddr >> 30) & 0x1ff
    ];

    let mut v = &root.entries[vpn[2]];

    for i in (0..=2).rev() {

        if v.is_invalid() {
            break;
        }
        else if v.is_leaf() {
            let off_mask = (1 << (12 + i * 9)) - 1;
            let vaddr_pgoff = vaddr & off_mask;
            let addr = ((v.get_entry() << 2) as usize) & !off_mask;
            return Some(addr | vaddr_pgoff);
        }

        let entry = ((v.get_entry() & !0x3ff) << 2) as *const Entry;
        v = unsafe { entry.add(vpn[i - 1]).as_ref().unwrap() };
    }

    None

}