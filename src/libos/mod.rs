pub mod api;
pub mod context;
pub mod debug;
pub mod page;
pub mod mem;
pub mod vm;
pub mod mock_mem;
pub mod addr;
use crate::utils::init_once::InitOnce;
use page::PageAlloc;

#[no_mangle]
fn main() {
    unsafe { api::_main_for_arch(0) };
}

pub(crate) static PAGE_ALLOC: InitOnce<&dyn PageAlloc> = InitOnce::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_once_by(page_alloc);
}