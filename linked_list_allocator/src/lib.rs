#![no_std]
#![feature(const_mut_refs)]
#![feature(allocator_api)]

/// Wrapper over [`spin::Mutex`].
pub mod mutex;

use core::{
    alloc::{AllocError, Allocator, Layout},
    mem,
    ptr::NonNull,
};

use common::{
    align::{align_to, size_align},
    convert::slice_ptr_from_mem_pos,
    mmap::allocate_bytes,
};

use mutex::Mutexed;

pub type LockedLinkedListAllocator = Mutexed<LinkedListAllocator>;
pub type Result<T> = core::result::Result<T, AllocError>;

impl LockedLinkedListAllocator {
    /// Creates a [`LockedBumpAllocator`] allocating the given amount of bytes.
    pub fn with_capacity(cap: usize) -> Result<Self> {
        LinkedListAllocator::with_capacity(cap).map(Mutexed::new)
    }
}

const NODE_SIZE: usize = mem::size_of::<Node>();
const NODE_ALIGN: usize = mem::align_of::<Node>();

/// A node in the free list
struct Node {
    /// The size of the memory region
    /// this node represents
    size: usize,
    /// A pointer to the next node, if there's one
    next: Option<&'static mut Node>,
}

unsafe impl Allocator for LockedLinkedListAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>> {
        let mut alloc = self.lock();

        let (size, align) = size_align::<Node>(layout);

        match alloc.find_region(size, align) {
            Some((region, alloc_start)) => {
                let alloc_end = alloc_start.checked_add(size).ok_or(AllocError)?;
                let excess_size = region.ending_address() - alloc_end;
                if excess_size > 0 {
                    unsafe {
                        alloc.add_free_region(alloc_end, excess_size);
                    }
                }

                unsafe { slice_ptr_from_mem_pos(alloc_start, size) }.ok_or(AllocError)
            }
            None => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, non_null: NonNull<u8>, layout: Layout) {
        let (size, _) = size_align::<Node>(layout);

        self.lock()
            .add_free_region(non_null.as_ptr() as usize, size)
    }
}

impl Node {
    const fn with_size(size: usize) -> Self {
        Self { size, next: None }
    }

    /// Checks if this node can allocate the
    /// given size and alignment.
    ///
    /// Returns the allocation start address on success
    ///
    /// Panics if `align` is not a power of two
    pub fn can_allocate(&self, size: usize, align: usize) -> Option<usize> {
        let alloc_start = align_to(self.starting_address(), align);
        let alloc_end = alloc_start.checked_add(size)?;

        if alloc_end > self.ending_address() {
            // This node cannot fit `size`
            return None;
        }

        // How much memory would be left in the node after fitting size
        let excess_size = self.ending_address().checked_sub(alloc_end)?;
        if excess_size < NODE_SIZE {
            return None;
        }

        Some(alloc_start)
    }

    fn starting_address(&self) -> usize {
        self as *const Self as usize
    }

    fn ending_address(&self) -> usize {
        self.starting_address() + self.size
    }
}

pub struct LinkedListAllocator {
    head: Node,
}

impl LinkedListAllocator {
    pub fn with_capacity(cap: usize) -> Result<Self> {
        let allocated = NonNull::new(allocate_bytes(cap)).ok_or(AllocError)?;
        let start = allocated.as_ptr() as usize;

        // Safety: mmap was called with `MAP_PRIVATE` meaning the memory returned
        //         to us is currently unused
        Ok(unsafe { Self::new(start, start + cap) })
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the given
    /// heap bounds are valid.
    pub unsafe fn new(heap_start: usize, heap_size: usize) -> Self {
        let mut this = Self {
            head: Node::with_size(0),
        };

        this.add_free_region(heap_start, heap_size);
        this
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // Ensure that the free region is capable of holding a Node
        assert!(size >= NODE_SIZE);
        // Make sure `addr` is properly aligned to `Node`
        assert_eq!((addr as *const u8).align_offset(NODE_ALIGN), 0);
        assert_eq!(align_to(addr, NODE_ALIGN), addr);

        // create a new list node and append it at the start of the list
        let mut new_node = Node::with_size(size);

        new_node.next = self.head.next.take();
        let node_ptr = addr as *mut Node;

        node_ptr.write(new_node);
        self.head.next = Some(&mut *node_ptr)
    }

    /// Looks for a free region with the given size and alignment and removes
    /// it from the list.
    ///
    /// Returns a tuple of the list node and the start address of the allocation.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut Node, usize)> {
        // reference to current list node, updated for each iteration
        let mut current = &mut self.head;
        // look for a large enough memory region in linked list
        while let Some(ref mut region) = current.next {
            match region.can_allocate(size, align) {
                Some(alloc_start) => {
                    // region suitable for allocation -> remove node from list
                    let next = region.next.take();
                    let ret = Some((current.next.take().unwrap(), alloc_start));
                    current.next = next;
                    return ret;
                }
                None => {
                    current = current.next.as_mut()?;
                }
            }
        }

        // no suitable region found
        None
    }
}
