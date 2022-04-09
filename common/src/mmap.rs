use libc::{c_void, mmap, munmap, MAP_ANON, MAP_PRIVATE, PROT_READ, PROT_WRITE};

/// Request a contiguous and anonymous memory block from the kernel
pub fn allocate_bytes(bytes: usize) -> *mut c_void {
    unsafe {
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
    }
}

/// Unmaps a memory block previously returned by [`mmap`] starting
/// at `pos` until `pos + bytes - 1`.
///
/// Returns true if the operation succeeds
pub fn deallocate_bytes(pos: usize, bytes: usize) -> bool {
    0 == unsafe { munmap(pos as *mut _, bytes) }
}
