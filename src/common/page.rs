use crate::PhysPage;

pub trait PageAlloc: Sync {
    fn alloc(&self) -> PhysPage;
    fn dealloc(&self, ppn: PhysPage);
}