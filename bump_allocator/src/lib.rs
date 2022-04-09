#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(allocator_api)]

/// Wrapper over [`spin::Mutex`].
pub mod mutex;

use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use common::convert::slice_ptr_from_mem_pos;
use common::{
    align::align_to,
    mmap::{allocate_bytes, deallocate_bytes},
};
use mutex::Mutexed;

pub type Result<T> = core::result::Result<T, AllocError>;

/// A single-block bump heap allocator protected by a spinlock mutex.
pub type LockedBumpAllocator = Mutexed<BumpAllocator>;

impl LockedBumpAllocator {
    /// Creates a [`LockedBumpAllocator`] allocating the given amount of bytes.
    pub fn with_capacity(cap: usize) -> Result<Self> {
        BumpAllocator::with_capacity(cap).map(Mutexed::new)
    }
}

unsafe impl Allocator for LockedBumpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>> {
        let mut alloc = self.lock();

        // The start of the current allocation
        let start = align_to(alloc.next, layout.align());
        // The end of the current allocation
        let end = start.checked_add(layout.size()).ok_or(AllocError)?;

        // Check if we've run out of memory
        (end <= alloc.heap_end).then(|| {}).ok_or(AllocError)?;

        alloc.next = end;
        alloc.allocations += 1;

        unsafe { slice_ptr_from_mem_pos(start, layout.size()) }.ok_or(AllocError)
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {
        let mut alloc = self.lock();

        alloc.allocations -= 1;
        if alloc.allocations == 0 {
            alloc.next = alloc.heap_start;
        }
    }
}

/// Single-block bump heap allocator
pub struct BumpAllocator {
    /// The upper bound (or start) of the heap memory region.
    heap_start: usize,
    /// The lower bound (or end) of the heap memory region.
    heap_end: usize,
    /// The next unused memory position in the heap.
    next: usize,
    /// How many allocations were made
    allocations: usize,
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let ok = deallocate_bytes(self.heap_start, self.heap_end - self.heap_start);

        debug_assert!(ok)
    }
}

impl BumpAllocator {
    pub fn with_capacity(cap: usize) -> Result<Self> {
        let ptr = NonNull::new(allocate_bytes(cap)).ok_or(AllocError)?;

        let start = ptr.as_ptr() as usize;

        // Safety: mmap was called with `MAP_PRIVATE` meaning the memory returned
        //         to us is currently unused
        Ok(unsafe { Self::new(start, start + cap) })
    }

    /// Creates a [`BumpAllocator`] with the given heap region
    ///
    /// # Safety
    ///
    /// Caller must ensure that the memory range is not being used
    unsafe fn new(start: usize, end: usize) -> Self {
        Self {
            heap_start: start,
            heap_end: end,
            next: start,
            allocations: 0,
        }
    }
}
