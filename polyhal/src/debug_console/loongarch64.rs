use ns16550a::Uart;
use spin::Mutex;

use super::DebugConsole;

#[cfg(not(board = "2k1000"))]
const UART_ADDR: usize = 0x01FE001E0 | crate::arch::consts::VIRT_ADDR_START;
#[cfg(board = "2k1000")]
const UART_ADDR: usize = 0x800000001fe20000;
// 0x800000001fe20000ULL
static COM1: Mutex<Uart> = Mutex::new(Uart::new(UART_ADDR));

impl DebugConsole {
    /// Writes a byte to the console.
    #[inline]
    pub fn putchar(ch: u8) {
        if ch == b'\n' {
            COM1.lock().put(b'\r');
        }
        COM1.lock().put(ch);
    }

    /// read a byte, return -1 if nothing exists.
    #[inline]
    pub fn getchar() -> Option<u8> {
        COM1.lock().get()
    }
}
