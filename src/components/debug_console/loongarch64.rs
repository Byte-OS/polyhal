use spin::Mutex;

use crate::components::debug_console::DebugConsole;

#[cfg(not(board = "2k1000"))]
const UART_ADDR: usize = 0x01FE001E0 | crate::components::consts::VIRT_ADDR_START;
#[cfg(board = "2k1000")]
const UART_ADDR: usize = 0x800000001fe20000;
// 0x800000001fe20000ULL
static COM1: Mutex<Uart> = Mutex::new(Uart::new(UART_ADDR));

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub const fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    pub fn putchar(&mut self, c: u8) {
        let ptr = self.base_address as *mut u8;
        loop {
            unsafe {
                if ptr.add(5).read_volatile() & (1 << 5) != 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    pub fn getchar(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

impl DebugConsole {
    /// Writes a byte to the console.
    pub fn putchar(ch: u8) {
        if ch == b'\n' {
            COM1.lock().putchar(b'\r');
        }
        COM1.lock().putchar(ch)
    }

    /// read a byte, return -1 if nothing exists.
    #[inline]
    pub fn getchar() -> Option<u8> {
        COM1.lock().getchar()
    }
}
