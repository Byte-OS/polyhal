use core::{alloc::Layout, cell::UnsafeCell, mem::size_of, ptr::copy_nonoverlapping};

use alloc::alloc::alloc;

use crate::{debug::println, PAGE_SIZE};

static BOOT_PERCPU_DATA_AREA: [u8; PAGE_SIZE] = [0; PAGE_SIZE];

/// Returns the base address of the per-CPU data area on the given CPU.
///
/// if `cpu_id` is 0, it returns the base address of all per-CPU data areas.
pub fn percpu_area_init(cpu_id: usize) -> usize {
    // Get initial per-CPU data area
    extern "Rust" {
        fn __start_percpu();
        fn __stop_percpu();
    }
    let start = __start_percpu as usize;
    let size = __stop_percpu as usize - start;
    // use polyhal_macro::percpu_symbol_offset;
    // let start = percpu_symbol_offset!(__start_percpu);
    // let size = percpu_symbol_offset!(__stop_percpu) - percpu_symbol_offset!(__start_percpu);

    println!("start: {:#x} size: {:#x}", start, size);

    // Get the base address of the per-CPU data area
    // If cpu_id is boot,core then use BOOT_PERCPU_DATA_AREA.
    // else alloc area.
    let dst = if cpu_id == 0 {
        BOOT_PERCPU_DATA_AREA.as_ptr() as usize as *mut u8
    } else {
        let layout =
            Layout::from_size_align(size, size_of::<usize>()).expect("percpu area align failed");
        unsafe { alloc(layout) }
    };

    // Init the area with original data.
    unsafe {
        copy_nonoverlapping(start as *const u8, dst, size);
    }

    dst as usize
}

/// Read the architecture-specific thread pointer register on the current CPU.
pub fn get_local_thread_pointer() -> usize {
    let tp;
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                tp = x86::msr::rdmsr(x86::msr::IA32_GS_BASE) as usize
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv {}, gp", out(reg) tp)
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("mrs {}, TPIDR_EL1", out(reg) tp)
            } else if #[cfg(target_arch = "loongarch64")] {
                core::arch::asm!("move {}, $r21", out(reg) tp)
            }
        }
    }
    tp
}

/// Set the architecture-specific thread pointer register to the per-CPU data
/// area base on the current CPU.
///
/// `cpu_id` indicates which per-CPU data area to use.
pub fn set_local_thread_pointer(cpu_id: usize) {
    let tp = percpu_area_init(cpu_id);
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                x86::msr::wrmsr(x86::msr::IA32_GS_BASE, tp as u64);
                SELF_PTR.write_current_raw(tp);
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv gp, {}", in(reg) tp)
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("msr TPIDR_EL1, {}", in(reg) tp)
            } else if #[cfg(target_arch = "loongarch64")] {
                core::arch::asm!("move $r21, {}", in(reg) tp)
            }
        }
    }
}

/// On x86, we use `gs:SELF_PTR` to store the address of the per-CPU data area base.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[polyhal_macro::def_percpu]
static SELF_PTR: usize = 0;

// 思路: 这里存放的 usize 数据，如果是 u8, u16, u32, u64, 或者 size 小于等于 u64 的直接存取
// 否则这里仅表示指针。
pub struct PerCpu<T> {
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for PerCpu<T> {}

impl<T> PerCpu<T> {
    pub const fn new(value: T) -> Self {
        PerCpu {
            value: UnsafeCell::new(value),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.value.get() }
    }

    pub fn write(&self, value: T) {
        unsafe {
            core::ptr::write_volatile(self.value.get(), value);
        }
    }

    // pub fn offset(&self) -> usize {
    //     extern "Rust" {
    //         #[link_name = "__start_per_cpu"]
    //         fn SECION_START();
    //     }
    //     self.value.get() as usize - SECION_START as usize
    // }
}

pub fn init_per_cpu() {}
