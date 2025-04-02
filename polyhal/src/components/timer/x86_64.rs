use raw_cpuid::CpuId;
use x86_64::structures::port::{self, PortRead};

use crate::{arch::apic::local_apic, time::Time};

static mut CPU_FREQ_MHZ: usize = 4_000_000_000;

impl Time {
    #[inline]
    pub fn get_freq() -> usize {
        unsafe { CPU_FREQ_MHZ }
    }

    #[inline]
    pub fn now() -> Self {
        Self(unsafe { core::arch::x86_64::_rdtsc() as _ })
    }
}

pub(crate) fn init_early() {
    if let Some(freq) = CpuId::new()
        .get_processor_frequency_info()
        .map(|info| info.processor_base_frequency())
    {
        if freq > 0 {
            log::info!("Got TSC frequency by CPUID: {} MHz", freq);
            unsafe { CPU_FREQ_MHZ = freq as _ }
        }
    }

    unsafe {
        use x2apic::lapic::{TimerDivide, TimerMode};
        let lapic = local_apic();
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_divide(TimerDivide::Div2); // indeed it is Div1, the name is confusing.
        lapic.enable_timer();

        // PIT(Programmable Interval Timer)
        // https://wiki.osdev.org/Pit
        // Bits         Usage
        // 6 and 7      Select channel :
        //                 0 0 = Channel 0
        //                 0 1 = Channel 1
        //                 1 0 = Channel 2
        //                 1 1 = Read-back command (8254 only)
        // 4 and 5      Access mode :
        //                 0 0 = Latch count value command
        //                 0 1 = Access mode: lobyte only
        //                 1 0 = Access mode: hibyte only
        //                 1 1 = Access mode: lobyte/hibyte
        // 1 to 3       Operating mode :
        //                 0 0 0 = Mode 0 (interrupt on terminal count)
        //                 0 0 1 = Mode 1 (hardware re-triggerable one-shot)
        //                 0 1 0 = Mode 2 (rate generator)
        //                 0 1 1 = Mode 3 (square wave generator)
        //                 1 0 0 = Mode 4 (software triggered strobe)
        //                 1 0 1 = Mode 5 (hardware triggered strobe)
        //                 1 1 0 = Mode 2 (rate generator, same as 010b)
        //                 1 1 1 = Mode 3 (square wave generator, same as 011b)
        // 0            BCD/Binary mode: 0 = 16-bit binary, 1 = four-digit BCD
        // open PIT2
        let pcspeaker = u8::read_from_port(0x61);
        u8::write_to_port(0x61, pcspeaker | 1);

        const PIT_FREQ: u16 = 11931;
        use port::PortWrite;
        // Set PIT2 one-shott mode
        u8::write_to_port(0x43, 0b10110010);

        // Write frequency to port
        u16::write_to_port(0x42, PIT_FREQ & 0xff);
        u16::write_to_port(0x42, (PIT_FREQ >> 8) & 0xff);

        // Reset PIT2 counter
        let pcspeaker = u8::read_from_port(0x61);
        u8::write_to_port(0x61, pcspeaker & 0xfd);
        u8::write_to_port(0x61, pcspeaker | 1);

        // Reset loapic counter
        lapic.set_timer_initial(0xFFFF_FFFF);

        // Read count
        loop {
            let mut count = u16::read_from_port(0x42);
            count |= u16::read_from_port(0x42) << 8;
            if count == 0 || count >= 60000 {
                break;
            }
        }
        let end = lapic.timer_current();

        let ticks10ms = 0xFFFF_FFFF - end;
        // lapic.set_timer_initial(0x20_000);
        // Set ticks 1s
        // lapic.set_timer_initial(ticks10ms * 0x100);
        // Set 500us ticks
        lapic.set_timer_initial(ticks10ms / 20);
        // set_oneshot_timer(2000);
    }
}
