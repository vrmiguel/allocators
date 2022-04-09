/// Check if the given number is a power of two.
pub fn is_power_of_two(x: usize) -> bool {
    (x != 0) && (x & (x - 1)) == 0
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
pub fn align_to(addr: usize, align: usize) -> usize {
    debug_assert!(is_power_of_two(align));

    (addr + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use core::ops::Not;

    use crate::align::{align_to, is_power_of_two};

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

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(16));
        assert!(is_power_of_two(128));

        assert!(is_power_of_two(0).not());
        assert!(is_power_of_two(3).not());
        assert!(is_power_of_two(8 + 16).not());
        assert!(is_power_of_two(8 + 32).not());
        assert!(is_power_of_two(16 + 64).not());
    }
}
