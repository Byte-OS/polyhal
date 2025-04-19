//! Uart 16550.
use crate::arch::get_com_port;
use crate::utils::MutexNoIrq;
use uart_16550::SerialPort;

static COM1: MutexNoIrq<SerialPort> = MutexNoIrq::new(unsafe { SerialPort::new(0x2f8) });

pub(crate) fn init() {
    // FIXME: Use dynamic port
    if let Some(port) = get_com_port(1) {
        *COM1.lock() = unsafe { SerialPort::new(port) };
    }
    COM1.lock().init();
}

#[inline]
pub fn putchar(c: u8) {
    COM1.lock().send(c);
}

#[inline]
pub fn getchar() -> Option<u8> {
    COM1.lock().try_receive().ok()
}
