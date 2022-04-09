#![no_std]
#![feature(allocator_api)]

use core::{alloc::{AllocError, Allocator, Layout}, ptr::NonNull};

use common::align::align_to;
use common::mmap::allocate_bytes;
use mutex::Mutexed;

pub type LockedBumpAllocator = Mutexed<BumpAllocator>;

impl LockedBumpAllocator {
    pub fn with_capacity(cap: usize) -> Result<Self> {
        BumpAllocator::with_capacity(cap).map(Mutexed::new)
    }
}

/// Wrapper over [`spin::Mutex`].
pub mod mutex;

pub type Result<T> = core::result::Result<T, AllocError>;

unsafe impl Allocator for LockedBumpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>> {
        let mut alloc = self.lock();

        let start = align_to(alloc.next, layout.align());
        let end = start.checked_add(layout.size()).ok_or(AllocError)?;

        // Check if we've run out of memory
        (end <= alloc.end).then(|| {}).ok_or(AllocError)?;

        alloc.next = end;
        alloc.allocations += 1;

        let slice = unsafe {
            core::slice::from_raw_parts_mut(start as *mut u8, layout.size()) 
        };

        let ptr = NonNull::new(slice as *mut _).ok_or(AllocError)?;

        Ok(ptr)
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {
        let mut alloc = self.lock(); // get a mutable reference

        alloc.allocations -= 1;
        if alloc.allocations == 0 {
            alloc.next = alloc.start;
        }
    }
}

pub struct BumpAllocator {
    /// The upper bound (or start) of the heap memory region.
    start: usize,
    /// The lower bound (or end) of the heap memory region.
    end: usize,
    /// The next unused memory position in the heap.
    next: usize,
    /// How many allocations were made
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a [`BumpAllocator`] allocating the given amount of bytes.
    pub fn with_capacity(cap: usize) -> Result<Self> {
        let ptr = NonNull::new(allocate_bytes(cap)).ok_or(AllocError)?;

        let start = ptr.as_ptr() as usize;
        
        // Safety: mmap was called with `MAP_PRIVATE` meaning the memory returned
        //         to us is currently unused
        Ok(unsafe { Self::new(start, start + cap) })
    }

    /// Creates a [`BumpAllocator`] allocating the amount of bytes
    ///
    /// # Safety
    ///
    /// Caller must ensure that the memory range is not being used
    unsafe fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            next: start,
            allocations: 0,
        }
    }
}
