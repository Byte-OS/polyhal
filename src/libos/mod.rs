pub mod addr;
pub mod context;
pub mod debug;
pub mod mem;
pub mod mock_mem;
pub mod vm;
use crate::debug::display_info;
use crate::utils::once::LazyInit;
use crate::PageAlloc;

extern "Rust" {
    pub(crate) fn _main_for_arch(hartid: usize);
}

#[no_mangle]
fn main() {
    display_info!();
    println!(include_str!("../common/banner.txt"));
    display_info!("Platform Name", "libos");
    display_info!("Boot HART ID", "{}", 0);
    display_info!();
    unsafe { _main_for_arch(0) };
}

pub(crate) static PAGE_ALLOC: LazyInit<&dyn PageAlloc> = LazyInit::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_by(page_alloc);

    self::mem::init_mock_mem();
}

pub fn shutdown() {
    std::process::exit(0);
}
