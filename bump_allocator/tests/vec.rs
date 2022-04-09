#![feature(allocator_api)]

use bump_allocator::LockedBumpAllocator;

#[test] 
fn allocates_vec_correctly() {
    let alloc = LockedBumpAllocator::with_capacity(24).unwrap();
    let mut bytes: Vec<u8, _> = Vec::new_in(alloc);

    bytes.push(2);
    bytes.push(3);
    bytes.push(4);
    
    assert_eq!(bytes, &[2, 3, 4])
}
