use core::alloc::Layout;
use core::mem;

/// Align the given address `addr` upwards to alignment `align`.
///
/// Panics if `align` is not a power of two.
pub fn align_to(addr: usize, align: usize) -> usize {
    let offset = (addr as *const u8).align_offset(align);
    addr + offset
}

/// Adjust the given layout so that the resulting allocated memory
/// region is also capable of storing an element of `T`.
///
/// Returns the adjusted size and alignment as a (size, align) tuple.
pub fn size_align<T>(layout: Layout) -> (usize, usize) {
    let layout = layout
        .align_to(mem::align_of::<T>())
        .expect("adjusting alignment failed")
        .pad_to_align();

    let size = layout.size().max(mem::size_of::<T>());

    (size, layout.align())
}

#[cfg(test)]
mod tests {
    use crate::align::align_to;

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(2, 8), 8);
        assert_eq!(align_to(7, 8), 8);
        assert_eq!(align_to(8, 8), 8);
        assert_eq!(align_to(11, 8), 16);
        assert_eq!(align_to(16, 8), 16);
        assert_eq!(align_to(255, 64), 256);
        assert_eq!(align_to(257, 64), 320);
    }
}
