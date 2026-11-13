use std::{
    pin::Pin,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

pub struct ItemAllocator<Item> {
    state: Arc<Mutex<AllocatorState<Item>>>,
}

impl<Item> Clone for ItemAllocator<Item> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<Item> ItemAllocator<Item> {
    pub fn new(item_count: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(AllocatorState::new(item_count))),
        }
    }
}

unsafe impl<Item> core::alloc::Allocator for ItemAllocator<Item> {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let mut state = self.state.lock().map_err(|_| std::alloc::AllocError)?;
        let location = state.reserve(layout);
        Ok(location)
    }

    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {}
}

struct AllocatorState<Item> {
    blocks: Vec<Pin<Box<[u8]>>>,
    next_idx: usize,
    item_count: usize,
    _p: std::marker::PhantomData<Item>,
}

impl<Item> AllocatorState<Item> {
    fn reserve(&mut self, layout: std::alloc::Layout) -> NonNull<[u8]> {
        let size = layout.size();
        // SAFETY:
        // `bytes` is init with one item.
        let block_bytes = unsafe { self.blocks.last_mut().unwrap_unchecked() };

        debug_assert!(self.next_idx + size <= block_bytes.len());
        let pos = &mut block_bytes[self.next_idx..self.next_idx + size];
        let pos = NonNull::from_ref(pos);

        self.next_idx += size;
        if self.next_idx >= block_bytes.len() {
            self.blocks
                .push(allocate_memory_block::<Item>(self.item_count));
            self.next_idx = 0;
        }
        pos
    }

    pub fn new(item_count: usize) -> Self {
        let mut bytes = Vec::with_capacity(1);
        bytes.push(allocate_memory_block::<Item>(item_count));
        Self {
            blocks: bytes,
            next_idx: 0,
            item_count,
            _p: Default::default(),
        }
    }
}

fn allocate_memory_block<Item>(item_count: usize) -> Pin<Box<[u8]>> {
    let byte_len = std::mem::size_of::<Item>() * item_count;
    let mut first_block = Vec::<u8>::with_capacity(byte_len);
    // SAFETY:
    // 1. The vector has a size of atleast `byte_len` bytes.
    // 2. Allocators are allowed to return uninit memory.
    unsafe {
        first_block.set_len(byte_len);
    };

    let boxed = first_block.into_boxed_slice();
    Pin::new(boxed)
}

#[cfg(test)]
mod tests {
    use std::{alloc::Layout, pin::Pin};

    use crate::allocator::AllocatorState;

    #[test]
    fn allocate_memory_block_u8() {
        use super::allocate_memory_block;
        const SIZE: usize = 256;
        const TYPE_LEN: usize = std::mem::size_of::<u8>();
        let block = allocate_memory_block::<u8>(SIZE);
        assert!(block.len() >= SIZE * TYPE_LEN)
    }

    #[test]
    fn allocate_memory_block_u32() {
        use super::allocate_memory_block;
        const SIZE: usize = 256;
        const TYPE_LEN: usize = std::mem::size_of::<u32>();
        let block = allocate_memory_block::<u32>(SIZE);
        assert!(block.len() >= SIZE * TYPE_LEN)
    }

    #[test]
    fn default_state_has_pre_reserved_memory() {
        let state = AllocatorState::<u8>::new(1);
        assert_eq!(state.blocks.len(), 1);
    }

    #[test]
    fn reserve_first_block_in_state() {
        let mut state = AllocatorState::<u8>::new(256);
        place_item_at_start(&mut state);
    }

    #[test]
    fn fill_block_in_state() {
        const COUNT: usize = 4;
        let mut state = AllocatorState::<u8>::new(COUNT);
        for next_empty in (1..COUNT).chain(std::iter::once(0)) {
            _ = state.reserve(Layout::new::<u8>());
            assert_eq!(state.next_idx, next_empty)
        }
        assert_eq!(state.blocks.len(), 2);

        place_item_at_start(&mut state);
    }

    /// This function tests if the first item in a block as been added at the first position.
    fn place_item_at_start<Item>(state: &mut AllocatorState<Item>) {
        let mem = state.reserve(Layout::new::<u8>());
        // the first allocation must point at the start
        let mem_ptr = mem.as_ptr() as *const [u8];
        let direct_ptr = {
            let last_block = state.blocks.last().unwrap();
            let data = Pin::as_ref(last_block);
            let data = &*data;
            data as *const [u8]
        };
        assert_eq!(mem_ptr.addr(), direct_ptr.addr());
        // The index must point to the next available entry
        assert_eq!(state.next_idx, 1);
    }
}
