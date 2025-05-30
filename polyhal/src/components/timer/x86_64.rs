use core::{arch::x86_64::_rdtsc, hint::spin_loop, time::Duration};

use raw_cpuid::CpuId;
use x2apic::lapic::{TimerDivide, TimerMode};
use x86_64::instructions::port::Port;

use crate::arch::apic::local_apic;

use super::current_time;

static mut CPU_FREQ_MHZ: u64 = 4_000_000_000;
const PIT_CH2_PORT: u16 = 0x42;
const PIT_CMD_PORT: u16 = 0x43;
const PC_SPEAKER_PORT: u16 = 0x61;

/// PIT(Programmable Interval Timer) frequency, 1ms
const PIT_FREQ: u16 = (1193182 / 1000) as u16;

/// Get ticks from system clock
///
/// # Return
///
/// - [u64] clock ticks
#[inline]
pub fn get_ticks() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() as _ }
}

/// Get frequency of the system clock
///
/// # Return
///
/// - [u64] n ticks per second
#[inline]
pub fn get_freq() -> u64 {
    unsafe { CPU_FREQ_MHZ }
}

/// Set the next timer
///
/// # parameters
///
/// - next [Duration] next time from system boot#[inline]
pub fn set_next_timer(next: Duration) {
    let curr = current_time();
    if next < curr {
        return;
    }
    let interval = next - curr;
    let lapic = local_apic();
    unsafe {
        lapic.set_timer_initial(
            (interval.as_secs() * get_freq()
                + interval.subsec_nanos() as u64 * get_freq() / 1_000_000_000) as _,
        );
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
        lapic.set_timer_divide(TimerDivide::Div1);
        lapic.enable_timer();

        let mut pc_speaker: Port<u8> = Port::new(PC_SPEAKER_PORT);
        let value = pc_speaker.read();
        pc_speaker.write(value & 0xfd); // clear bit 1

        // Reset lapic counter
        lapic.set_timer_initial(0xFFFF_FFFF);

        // Get CPU Frequency: (end - start) / 10ms
        let _start = _rdtsc();
        timer_wait(Duration::from_millis(10));
        let _end = _rdtsc();
        lapic.set_timer_mode(TimerMode::TscDeadline);
        lapic.set_timer_initial(0);
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
    let mut pc_speaker: Port<u8> = Port::new(PC_SPEAKER_PORT);
    let mut pit_ch2: Port<u8> = Port::new(PIT_CH2_PORT);
    let mut pit_cmd: Port<u8> = Port::new(PIT_CMD_PORT);
    unsafe {
        let mut value = pc_speaker.read();
        pc_speaker.write(value & !0x2);
        value = pc_speaker.read();
        pc_speaker.write(value | 1);
    }
    for _ in 0..duration.as_millis() {
        unsafe {
            // Set PIT2 one-shot mode
            pit_cmd.write(0b10110010);

            // Write frequency to port
            pit_ch2.write(PIT_FREQ as u8);
            pit_ch2.write((PIT_FREQ >> 8) as u8);
            while pc_speaker.read() & 0x20 == 0 {
                spin_loop();
            }
        }
    }
}
