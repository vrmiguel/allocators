#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

/// Wrappers over [`libc::mmap`] and [`libc::munmap`].
pub mod mmap;

/// Helpers for address alignment.
pub mod align;

/// Common type conversions
pub mod convert;
