use buddy_system_allocator::LockedFrameAllocator;
use polyhal::{addr::PhysPage, PAGE_SIZE};
use spin::Lazy;

static LOCK_FRAME_ALLOCATOR: Lazy<LockedFrameAllocator<32>> =
    Lazy::new(|| LockedFrameAllocator::new());

pub fn add_frame_range(mm_start: usize, mm_end: usize) {
    let start = mm_start / PAGE_SIZE;
    let end = mm_end / PAGE_SIZE;
    LOCK_FRAME_ALLOCATOR.lock().add_frame(start, end);
}

pub fn frame_alloc() -> PhysPage {
    let ppn = LOCK_FRAME_ALLOCATOR
        .lock()
        .alloc(1)
        .expect("can't find memory page");
    PhysPage::new(ppn)
}

pub fn frame_dealloc(ppn: PhysPage) {
    LOCK_FRAME_ALLOCATOR.lock().dealloc(ppn.as_num(), 1);
}
