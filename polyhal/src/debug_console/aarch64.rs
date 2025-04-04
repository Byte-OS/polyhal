//! PL011 UART.

use arm_pl011::Pl011Uart;

use crate::{debug_console::DebugConsole, pa, utils::MutexNoIrq, PhysAddr};

const UART_BASE: PhysAddr = pa!(0x0900_0000);

static UART: MutexNoIrq<Pl011Uart> = MutexNoIrq::new(Pl011Uart::new(UART_BASE.get_mut_ptr()));

/// Initialize the UART
pub fn init_early() {
    UART.lock().init();
}

impl DebugConsole {
    /// Writes a byte to the console.
    pub fn putchar(c: u8) {
        match c {
            b'\n' => {
                UART.lock().putchar(b'\r');
                UART.lock().putchar(b'\n');
            }
            c => UART.lock().putchar(c),
        }
    }

    /// Reads a byte from the console, or returns [`None`] if no input is available.
    pub fn getchar() -> Option<u8> {
        UART.lock().getchar()
    }
}

ph_ctor!(
    AARCH64_INIT_CONSOLE,
    crate::ctor::CtorType::HALDriver,
    init_early
);
