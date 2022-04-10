#![feature(allocator_api)]

use linked_list_allocator::LockedLinkedListAllocator;

#[test]
fn allocates() {
    let alloc = LockedLinkedListAllocator::with_capacity(24).unwrap();
    let mut vec: Vec<u8, _> = Vec::new_in(alloc);
    vec.push(5);
    vec.push(6);
    vec.push(9);

    vec.push(11);

    vec.push(122);

    assert_eq!(vec, &[5, 6, 9, 11, 122])
}

#[test]
fn fails_to_reserve_more_than_capacity() {
    let alloc = LockedLinkedListAllocator::with_capacity(24).unwrap();
    let mut vec: Vec<u8, _> = Vec::new_in(alloc);
    vec.try_reserve(64).unwrap_err();
}
