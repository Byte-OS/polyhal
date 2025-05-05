use core::{arch::x86_64::_rdtsc, hint::spin_loop, time::Duration};

use raw_cpuid::CpuId;
use x2apic::lapic::{TimerDivide, TimerMode};
use x86_64::instructions::port::Port;

use crate::{arch::apic::local_apic, time::Time};

static mut CPU_FREQ_MHZ: usize = 4_000_000_000;
static mut PIT_CMD: Port<u8> = Port::new(0x43);
static mut PIT_CH2: Port<u8> = Port::new(0x42);
static mut PC_SPEAKER: Port<u8> = Port::new(0x61);

/// PIT(Programmable Interval Timer) frequency, 1ms
const PIT_FREQ: u16 = (1193182 / 1000) as u16;

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

pub(crate) fn init() {
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
        let lapic = local_apic();
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_divide(TimerDivide::Div1); // indeed it is Div1, the name is confusing.
        lapic.enable_timer();

        let pcspeaker = PC_SPEAKER.read();
        PC_SPEAKER.write(pcspeaker & 0xfd); // clear bit 1

        // Reset lapic counter
        lapic.set_timer_initial(0xFFFF_FFFF);

        // Get CPU Frequency: (end - start) / 10ms
        let _start = _rdtsc();
        timer_wait(Duration::from_millis(10));
        let _end = _rdtsc();
    }
}

/// Wait for the timer to expire
#[inline]
pub(crate) fn timer_wait(duration: Duration) {
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

    // Reset PIT2 counter
    unsafe {
        PC_SPEAKER.write(PC_SPEAKER.read() & !0x2);
        PC_SPEAKER.write(PC_SPEAKER.read() | 1);
    }
    for _ in 0..duration.as_millis() {
        unsafe {
            // Set PIT2 one-shot mode
            PIT_CMD.write(0b10110010);

            // Write frequency to port
            PIT_CH2.write(PIT_FREQ as u8);
            PIT_CH2.write((PIT_FREQ >> 8) as u8);
            while PC_SPEAKER.read() & 0x20 == 0 {
                spin_loop();
            }
        }
    }
}
