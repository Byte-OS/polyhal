cfg_if::cfg_if! {
    if #[cfg(board = "k210")] {
        mod k210;
        pub use k210::*;
    } else if #[cfg(board = "qemu")] {
        mod qemu;
        pub use qemu::*;
    } else if #[cfg(board = "cv1811h")] {
        mod cv1811h;
        pub use cv1811h::*;
    } else if #[cfg(board = "visionfive2")] {
        use riscv::register::sstatus;
        pub const CLOCK_FREQ: usize = 12500000;
        static DEVICE_TREE: &[u8] = include_bytes!("jh7110-visionfive-v2.dtb");
        pub fn init_device(hartid: usize, _device_tree: usize) -> (usize, usize) {
            unsafe {
                sstatus::set_sum();
            }
            (hartid, DEVICE_TREE.as_ptr() as usize)
        }
    } else {
        // compile_error!("not support this board");
        pub const CLOCK_FREQ: usize = 12500000;

        pub fn init_device(hartid: usize, device_tree: usize) -> (usize, usize) {
            // warn!("use default board config");
            (hartid, device_tree)
        }
    }
}
