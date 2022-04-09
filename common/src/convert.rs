use core::ptr::NonNull;

/// Constructs a [`NonNull<[u8]>`](NonNull) for `len` bytes starting at `pos`.
///
/// # Safety
///
/// * Caller must ensure that `pos` is a valid and initialized position in memory.
/// * Caller must ensure that all `len` bytes following `pos` are properly initialized.
pub unsafe fn slice_ptr_from_mem_pos(pos: usize, len: usize) -> Option<NonNull<[u8]>> {
    let slice = unsafe { core::slice::from_raw_parts_mut(pos as *mut u8, len) };

    NonNull::new(slice as *mut _)
}
