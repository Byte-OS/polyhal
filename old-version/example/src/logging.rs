use core::fmt::{self, Write};

use polyhal::debug_console::DebugConsole;

pub struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buffer = [0u8; 4];
        for c in s.chars() {
            puts(c.encode_utf8(&mut buffer).as_bytes())
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Logger.write_fmt(args).unwrap();
}

#[inline]
pub fn puts(buffer: &[u8]) {
    // use the main uart if it exists.
    for i in buffer {
        DebugConsole::putchar(*i);
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::logging::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::logging::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
