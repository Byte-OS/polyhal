use buddy_system_allocator::FrameAllocator;
use polyhal::{pagetable::PAGE_SIZE, PhysAddr};
use spin::Mutex;

static LOCK_FRAME_ALLOCATOR: Mutex<FrameAllocator<32>> = Mutex::new(FrameAllocator::new());

pub fn add_frame_range(mm_start: usize, mm_end: usize) {
    extern "C" {
        fn _end();
    }
    let mm_start = if mm_start <= mm_end && mm_end > _end as usize {
        (_end as usize + PAGE_SIZE - 1) / PAGE_SIZE
    } else {
        mm_start / PAGE_SIZE
    };
    let mm_end = mm_end / PAGE_SIZE;
    LOCK_FRAME_ALLOCATOR.lock().add_frame(mm_start, mm_end);
}

pub fn frame_alloc(count: usize) -> PhysAddr {
    let ppn = LOCK_FRAME_ALLOCATOR
        .lock()
        .alloc(count)
        .expect("can't find memory page");
    PhysAddr::new(ppn << 12)
}

pub fn frame_dealloc(paddr: PhysAddr) {
    LOCK_FRAME_ALLOCATOR.lock().dealloc(paddr.raw() >> 12, 1);
}
