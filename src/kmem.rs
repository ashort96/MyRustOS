// Adam Short
// 08/02/2020

use crate::page::{align_val, zalloc, Table, PAGE_SIZE};
use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr::null_mut;

#[repr(usize)]
enum AllocListFlags {
    Taken = 1 << 63
}

impl AllocListFlags {
    pub fn val(self) -> usize {
        self as usize
    }
}

struct AllocList {
    pub flags_size: usize
}

impl AllocList {

    pub fn is_taken(&self) -> bool {
        self.flags_size & AllocListFlags::Taken.val() != 0
    }

    pub fn is_free(&self) -> bool {
        return !self.is_taken()
    }

    pub fn set_taken(&mut self) {
        self.flags_size |= AllocListFlags::Taken.val();
    }

    pub fn set_free(&mut self) {
        self.flags_size &= !AllocListFlags::Taken.val();
    }

    pub fn set_size(&mut self, sz: usize) {
        let k = self.is_taken();
        self.flags_size = sz & !AllocListFlags::Taken.val();
        if k {
            self.flags_size |= AllocListFlags::Taken.val();
        }
    }

    pub fn get_size(&self) -> usize {
        self.flags_size & !AllocListFlags::Taken.val()
    }

}

static mut KMEM_HEAD: *mut AllocList = null_mut();
static mut KMEM_ALLOC: usize = 0;
static mut KMEM_PAGE_TABLE: *mut Table = null_mut();

pub fn get_head() -> *mut u8 {
    unsafe { KMEM_HEAD as *mut u8 }
}

pub fn get_page_table() -> *mut Table {
    unsafe { KMEM_PAGE_TABLE as *mut Table }
}

pub fn get_num_allocations() -> usize {
    unsafe { KMEM_ALLOC }
}

pub fn init() {
    unsafe {
        let k_alloc = zalloc(64);
        assert!(!k_alloc.is_null());
        KMEM_ALLOC = 64;
        KMEM_HEAD = k_alloc as *mut AllocList;
        (*KMEM_HEAD).set_free();
        (*KMEM_HEAD).set_size(KMEM_ALLOC * PAGE_SIZE);
        KMEM_PAGE_TABLE = zalloc(1) as *mut Table;
    }
}

pub fn kzmalloc(sz: usize) -> *mut u8 {
    let size = align_val(sz, 3);
    let ret = kmalloc(size);

    if !ret.is_null() {
        for i in 0..size {
            unsafe { (*ret.add(i)) = 0; }
        }
    }
    ret
}

pub fn kmalloc(sz: usize) -> *mut u8 {
    unsafe {
        let size = align_val(sz, 3) + size_of::<AllocList>();
        let mut head = KMEM_HEAD;
        let tail = (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * PAGE_SIZE) as *mut AllocList;

        while head < tail {
            if (*head).is_free() && size <= (*head).get_size() {
                let chunk_size = (*head).get_size();
                let rem = chunk_size - size;
                (*head).set_taken();

                if rem > size_of::<AllocList>() {
                    let next = (head as *mut u8).add(size) as *mut AllocList;
                    (*next).set_free();
                    (*next).set_size(rem);
                    (*head).set_size(size);
                }
                else {
                    (*head).set_size(chunk_size);
                }
                return head.add(1) as *mut u8;
            }
            else {
                head = (head as *mut u8).add((*head).get_size()) as *mut AllocList;
            }
        }
    }
    null_mut()
}

pub fn kfree(ptr: *mut u8) {
    unsafe {
        if !ptr.is_null() {
            let p = (ptr as *mut AllocList).offset(-1);
            
            if (*p).is_taken() {
                (*p).set_free();
            }
            coalesce();
        }
    }
}

////////////////////////////////////////////////////////////////////////////
// Coalesce the freed pages into one chunk
////////////////////////////////////////////////////////////////////////////
pub fn coalesce() {
    unsafe {
        let mut head = KMEM_HEAD;
        let tail = (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * PAGE_SIZE) as *mut AllocList;

        while head < tail {
            let next = (head as *mut u8).add((*head).get_size()) as *mut AllocList;

            if (*head).get_size() == 0 {
                break;
            }
            else if next >= tail {
                break;
            }
            else if (*head).is_free() && (*next).is_free() {
                (*head).set_size((*head).get_size() + (*next).get_size());
            }

            head = (head as *mut u8).add((*head).get_size()) as *mut AllocList;

        }
    }
}

////////////////////////////////////////////////////////////////////////////
// DEBUGING PURPOSES
////////////////////////////////////////////////////////////////////////////
pub fn print_table() {
    unsafe {
        let mut head = KMEM_HEAD;
        let tail = (KMEM_HEAD as *mut u8).add(KMEM_ALLOC * PAGE_SIZE) as *mut AllocList;

        while head < tail {
            println!("{:p}: Length = {:<10} Taken = {}", head, (*head).get_size(), (*head).is_taken());
            head = (head as *mut u8).add((*head).get_size()) as *mut AllocList;
        }
    }
}

struct OsGlobalAlloc;

unsafe impl GlobalAlloc for OsGlobalAlloc {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        kzmalloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        kfree(ptr);
    }
}

#[global_allocator]

static GA: OsGlobalAlloc = OsGlobalAlloc {};

#[alloc_error_handler]

pub fn alloc_error(l: Layout) -> ! {
    panic!("Allocator failed to allocate {} bytes with {}-byte alignment.", l.size(), l.align());
}