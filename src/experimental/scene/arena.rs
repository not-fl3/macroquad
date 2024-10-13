//! Gleaned from https://github.com/ratel-rust/toolshed/blob/master/src/arena.rs
//! and than modified a lot.
//!
//! Module containing the `Arena` and `Uninitialized` structs. For convenience the
//! `Arena` is exported at the root of the crate.

use std::cell::Cell;
use std::mem::size_of;

const ARENA_BLOCK: usize = 64 * 1024;

/// An arena implementation that uses preallocated 64KiB pages for all allocations.
/// If a new allocation were to be pushed over the the boundaries of the page, a
/// new page is internally allocated first, thus this version of the arena can never
/// run out of memory unless the process runs out of heap altogether.
///
/// Allocating a type larger than the page size will result in a new heap allocation
/// just for that type separate from the page mechanism.
pub struct Arena {
    store: Cell<Vec<Vec<u8>>>,
    ptr: Cell<*mut u8>,
    offset: Cell<usize>,
}

impl Arena {
    /// Create a new arena with a single preallocated 64KiB page.
    pub fn new() -> Self {
        let mut store = vec![Vec::with_capacity(ARENA_BLOCK)];
        let ptr = store[0].as_mut_ptr();

        Arena {
            store: Cell::new(store),
            ptr: Cell::new(ptr),
            offset: Cell::new(0),
        }
    }

    pub fn alloc(&self, size: usize) -> *mut u8 {
        // This should be optimized away for size known at compile time.
        if size > ARENA_BLOCK {
            return self.alloc_bytes(size);
        }

        let size = match size % size_of::<usize>() {
            0 => size,
            n => size + (size_of::<usize>() - n),
        };

        let offset = self.offset.get();
        let cap = offset + size;

        if cap > ARENA_BLOCK {
            self.grow();

            self.offset.set(size);
            self.ptr.get()
        } else {
            self.offset.set(cap);
            unsafe { self.ptr.get().add(offset) }
        }
    }

    #[inline]
    fn alloc_byte_vec(&self, mut val: Vec<u8>) -> *mut u8 {
        let ptr = val.as_mut_ptr();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val);
        self.store.replace(temp);

        ptr
    }

    pub fn grow(&self) {
        let ptr = self.alloc_byte_vec(Vec::with_capacity(ARENA_BLOCK));
        self.ptr.set(ptr);
    }

    fn alloc_bytes(&self, size: usize) -> *mut u8 {
        self.alloc_byte_vec(Vec::with_capacity(size))
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn offset(&self) -> usize {
        self.offset.get()
    }
}

/// Akin to `CopyCell`: `Sync` is unsafe but `Send` is totally fine!
unsafe impl Send for Arena {}
