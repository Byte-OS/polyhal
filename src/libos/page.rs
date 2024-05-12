use crate::PhysAddr;

pub trait PageAlloc: Sync {
    fn alloc(&self) -> PhysAddr;
    fn dealloc(&self, ppn: PhysAddr);
}