use core::cmp;

use self::vectors::*;
use super::consts::{PIC_VECTOR_OFFSET, VIRT_ADDR_START};
use crate::utils::MutexNoIrq;
use spin::Once;
use x2apic::ioapic::{IoApic, RedirectionTableEntry};
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};
use x86_64::instructions::port::Port;

pub mod vectors {
    pub const APIC_TIMER_VECTOR: u8 = 0xf0;
    pub const APIC_SPURIOUS_VECTOR: u8 = 0xf1;
    pub const APIC_ERROR_VECTOR: u8 = 0xf2;
}

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 256;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = APIC_TIMER_VECTOR as usize;

const IO_APIC_BASE: u64 = 0xFEC0_0000;

static mut LOCAL_APIC: Option<LocalApic> = None;
static mut IS_X2APIC: bool = false;
static IO_APIC: Once<MutexNoIrq<IoApic>> = Once::new();

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
// pub fn register_handler(vector: usize, handler: crate::irq::IrqHandler) -> bool {
//     crate::irq::register_handler_common(vector, handler)
// }

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
// pub fn dispatch_irq(vector: usize) {
//     crate::irq::dispatch_irq_common(vector);
//     unsafe { local_apic().end_of_interrupt() };
// }

pub fn local_apic<'a>() -> &'a mut LocalApic {
    // It's safe as LAPIC is per-cpu.
    unsafe { LOCAL_APIC.as_mut().unwrap() }
}

/// Get the interrupt controller
pub fn io_apic<'a>() -> &'a MutexNoIrq<IoApic> {
    IO_APIC.get().expect("Can't get io_apic")
}

pub fn raw_apic_id(id_u8: u8) -> u32 {
    if unsafe { IS_X2APIC } {
        id_u8 as u32
    } else {
        (id_u8 as u32) << 24
    }
}

/// Check if the current cpu supports the x2apic
fn cpu_has_x2apic() -> bool {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.has_x2apic(),
        None => false,
    }
}

/// PIC End of interrupt
/// 8259 Programmable Interrupt Controller
#[allow(dead_code)]
pub fn pic_eoi() {
    unsafe {
        Port::<u8>::new(0x20).write(0x20);
        Port::<u8>::new(0xa0).write(0x20);
    }
}

/// Init APIC
pub fn init() {
    // Remap and init pic controller.
    unsafe {
        let mut pic1_command = Port::<u8>::new(0x20);
        let mut pic1_data = Port::<u8>::new(0x21);
        let mut pic2_command = Port::<u8>::new(0xa0);
        let mut pic2_data = Port::<u8>::new(0xa1);
        // Remap 8259a master irqs to 0x20, slave to 0x28
        // Map PIC_IRQ -> PIC_IRQ_OFFSET(0x20)
        pic1_command.write(0x11);
        pic2_command.write(0x11);
        pic1_data.write(PIC_VECTOR_OFFSET);
        pic2_data.write(PIC_VECTOR_OFFSET + 8);
        pic1_data.write(0x04);
        pic2_data.write(0x02);
        pic1_data.write(0x01);
        pic2_data.write(0x01);

        // Disable 8259A interrupt controllers
        pic1_data.write(0xff);
        pic2_data.write(0xff);
    }
    let mut builder = LocalApicBuilder::new();
    builder
        .timer_vector(APIC_TIMER_VECTOR as _)
        .error_vector(APIC_ERROR_VECTOR as _)
        .spurious_vector(APIC_SPURIOUS_VECTOR as _);

    if cpu_has_x2apic() {
        unsafe { IS_X2APIC = true };
    } else {
        builder.set_xapic_base(unsafe { xapic_base() } | VIRT_ADDR_START as u64);
    }

    let mut lapic = builder.build().unwrap();
    unsafe {
        lapic.enable();
        LOCAL_APIC = Some(lapic);
    }

    let mut io_apic = unsafe { IoApic::new(IO_APIC_BASE) };
    // Remap the PIC irqs, Default disabled.
    for irq in 0..cmp::min(unsafe { io_apic.max_table_entry() }, 0x10) {
        let mut entry = RedirectionTableEntry::default();
        entry.set_vector(0x20 + irq);
        entry.set_dest(0); // CPU(s)

        // Set table entry and set it disabled.
        unsafe {
            io_apic.set_table_entry(irq, entry);
            io_apic.disable_irq(irq);
        }
    }

    // Initialize the IO_APIC
    IO_APIC.call_once(|| MutexNoIrq::new(io_apic));
}
