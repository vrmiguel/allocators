use core::ptr;
use libc::{c_void, mmap, munmap, MAP_ANON, MAP_PRIVATE, MREMAP_MAYMOVE, PROT_READ, PROT_WRITE};

/// Request a contiguous and anonymous memory block from the kernel.
///
/// Returns a null pointer if the operation failed.
pub fn allocate_bytes(bytes: usize) -> *mut c_void {
    let ptr = unsafe {
        mmap(
            // Let kernel decide where to place it in virtual address space
            core::ptr::null_mut(),
            // How much memory to be allocated, in bytes
            bytes,
            // Allocate read/write memory
            PROT_READ | PROT_WRITE,
            // Memory is zero-filled
            // Memory is used only by this process
            MAP_ANON | MAP_PRIVATE,
            // The following two arguments are not used for anonymous memory
            -1,
            0,
        )
    };

    match ptr {
        libc::MAP_FAILED => ptr::null_mut(),
        valid_ptr => valid_ptr,
    }
}

/// Unmaps a memory block previously returned by [`mmap`] starting
/// at `pos` until `pos + bytes - 1`.
///
/// Returns true if the operation succeeds.
pub fn deallocate_bytes(pos: usize, bytes: usize) -> bool {
    0 == unsafe { munmap(pos as *mut _, bytes) }
}

/// Attempt to reallocate a previously `mmap`'ed address (such as a pointer returned by [`allocate_bytes`]).
pub unsafe fn reallocate_bytes(
    old_addr: *mut c_void,
    old_size: usize,
    new_size: usize,
    can_relocate: bool,
) -> Option<*mut c_void> {
    // If MREMAP_MAYMOVE is set, the kernel is allowed to
    // relocate the previous mapping. If it's not set, we know the previous
    // mapping will be kept valid.
    let flags = if can_relocate { MREMAP_MAYMOVE } else { 0 };

    let ret_ptr = unsafe { libc::mremap(old_addr, old_size, new_size, flags) };

    match ret_ptr {
        libc::MAP_FAILED => None,
        valid_ptr => Some(valid_ptr),
    }
}

pub struct AnonymousMapping {
    start: usize,
    end: usize,
}

impl AnonymousMapping {
    pub fn bytes(&self) -> usize {
        self.end - self.start
    }

    // TODO: rest
}