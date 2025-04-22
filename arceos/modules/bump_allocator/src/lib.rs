#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    last_b_alloc: usize,
    last_b_pos: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        EarlyAllocator {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            last_b_alloc: 0,
            last_b_pos: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.add_memory(start, size);
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        Result::Ok(())
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: core::alloc::Layout) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let align = layout.align();
        let b_pos = (self.b_pos + align - 1) & !(align - 1);
        if b_pos + layout.size() > self.p_pos {
            return Result::Err(allocator::AllocError::NoMemory);
        }
        let ptr = core::ptr::NonNull::new(self.b_pos as *mut u8).unwrap();
        self.last_b_alloc = b_pos;
        self.last_b_pos = self.b_pos;
        self.b_pos += b_pos + layout.size();
        Result::Ok(ptr)
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if self.last_b_alloc != pos.as_ptr() as usize {
            return;
        }
        self.b_pos = self.last_b_pos;
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;
    
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> allocator::AllocResult<usize> {
        let align = 1 << align_pow2;
        let p_pos = (self.p_pos - num_pages * Self::PAGE_SIZE + align - 1) & !(align - 1);
        if p_pos < self.end {
            return Result::Err(allocator::AllocError::NoMemory);
        }
        self.p_pos = p_pos;
        Result::Ok(p_pos)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {}


    fn available_pages(&self) -> usize {
        (self.p_pos - self.end) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / Self::PAGE_SIZE
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / Self::PAGE_SIZE
    }
}
