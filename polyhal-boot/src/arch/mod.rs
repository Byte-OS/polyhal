use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use polyhal::{
    common::get_cpu_num,
    ctor::{ph_init_iter, CtorType},
    println,
};

// Define multi-architecture modules and pub use them.
cfg_if::cfg_if! {
    if #[cfg(target_arch = "loongarch64")] {
        mod loongarch64;
    } else if #[cfg(target_arch = "aarch64")] {
        mod aarch64;
    } else if #[cfg(target_arch = "riscv64")] {
        mod riscv64;
    } else if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
    } else {
        compile_error!("unsupported architecture!");
    }
}

/// Clear the bss section
pub(crate) fn clear_bss() {
    extern "C" {
        fn _sbss();
        fn _ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            _sbss as usize as *mut u128,
            (_ebss as usize - _sbss as usize) / size_of::<u128>(),
        )
        .fill(0);
    }
}

fn call_real_main(hartid: usize) {
    // polyhal::multicore::boot_core(cpuid, addr, sp_top);
    static IS_BOOT: AtomicBool = AtomicBool::new(true);
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    extern "Rust" {
        fn _secondary_start();
        pub(crate) fn _main_for_arch(hartid: usize);
        pub(crate) fn _secondary_for_arch(hartid: usize);
    }

    if IS_BOOT.swap(false, Ordering::SeqCst) {
        const SP_SIZE: usize = 0x40_0000;

        (0..get_cpu_num()).for_each(|x| unsafe {
            if x == hartid {
                return;
            }
            let stack_top = polyhal::mem::alloc(SP_SIZE).add(SP_SIZE);
            println!("Boot Core: {}   {:#p}", x, stack_top);
            polyhal::multicore::boot_core(x, _secondary_start as usize, stack_top as usize);
        });
        polyhal::println!();

        // Run Kernel's Contructors Before Droping Into Kernel.
        ph_init_iter(CtorType::KernelService).for_each(|x| (x.func)());
        ph_init_iter(CtorType::Normal).for_each(|x| (x.func)());
        INIT_DONE.store(true, Ordering::SeqCst);
        // Declare the _main_for_arch exists.
        unsafe {
            _main_for_arch(hartid);
        }
    } else {
        while !INIT_DONE.load(Ordering::SeqCst) {
            spin_loop();
        }
        unsafe {
            _secondary_for_arch(hartid);
        }
    }
    loop {
        spin_loop();
    }
}
