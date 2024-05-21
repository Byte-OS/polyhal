pub mod context;
pub mod debug;
pub mod mem;
pub mod vm;
pub mod mock_mem;
pub mod addr;
use crate::utils::once::LazyInit;
use crate::PageAlloc;

extern "Rust" {
    pub(crate) fn _main_for_arch(hartid: usize);
}

#[no_mangle]
fn main() {
    unsafe { _main_for_arch(0) };
}

pub(crate) static PAGE_ALLOC: LazyInit<&dyn PageAlloc> = LazyInit::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_by(page_alloc);

    self::mem::init_mock_mem();
}