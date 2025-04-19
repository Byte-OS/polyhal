//! This is a console for debugging,
//! If you want to use this logging
//! You need to use like this:
//!
//! #### Put a char to output device(always uart)
//! ```rust
//! DebugConsole::putchar(b'3');
//! ```
//!
//! ### Get a char from input device(always uart)
//! ```rust
//! DebugConsole::getchar();
//! ```
polyhal_macro::define_arch_mods!();

use core::fmt::Write;

use crate::utils::MutexNoIrq;

/// Print macro to print polyhal information with newline
#[macro_export]
macro_rules! println {
    () => {
        $crate::debug_console::_print(format_args!("\n"))
    };
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::debug_console::_print(format_args!("{}\n", format_args!($fmt $(, $($arg)+)?)))
    };
}

/// Print macro to print polyhal information with newline
#[macro_export]
macro_rules! print {
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::debug_console::_print(format_args!($fmt $(, $($arg)+)?))
    };
}

/// Display Platform Information with specified format
/// display_info!("item name", "{}", "format");
/// The output format like below:
/// item name             : format
#[macro_export]
macro_rules! display_info{
    () => {
        $crate::debug_console::_print(format_args!("\n"))
    };
    ($item:expr,$fmt: expr $(, $($arg: tt)+)?) => {
        $crate::debug_console::_print(format_args!("{:<26}: {}\n", $item, format_args!($fmt $(, $($arg)+)?)))
    };
}

/// Print the given arguments
#[inline]
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    static LOCK: MutexNoIrq<()> = MutexNoIrq::new(());
    let lock = LOCK.lock();
    DebugConsole.write_fmt(args).expect("can't print arguments");
    drop(lock)
}

pub struct DebugConsole;

// Write string through DebugConsole
impl Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|&x| Self::putchar(x));
        Ok(())
    }
}

#[cfg(feature = "logger")]
impl log::Log for DebugConsole {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use log::Level;

        let file = record.module_path();
        let line = record.line();
        #[cfg(all(target_arch = "x86_64", feature = "graphic"))]
        {
            let color_code = match record.level() {
                Level::Error => 0xff0000u32, // Red
                Level::Warn => 0xFFFF00,     // BrightYellow
                Level::Info => 0x33ccff,     // Blue
                Level::Debug => 0x00ff00,    // Green
                Level::Trace => 0xaaaaaa,    // BrightBlack
            };
            DebugConsole::set_color(color_code);
            println!(
                "[{}] <{}:{}> {}",
                record.level(),
                file.unwrap(),
                line.unwrap(),
                record.args()
            );
            DebugConsole::set_color(0xffffff);
        }
        #[cfg(not(all(target_arch = "x86_64", feature = "graphic")))]
        {
            let color_code = match record.level() {
                Level::Error => 31u8, // Red
                Level::Warn => 93,    // BrightYellow
                Level::Info => 34,    // Blue
                Level::Debug => 32,   // Green
                Level::Trace => 90,   // BrightBlack
            };
            println!(
                "\u{1B}[{}m\
                    [{}] <{}:{}> {}\
                    \u{1B}[0m",
                color_code,
                record.level(),
                file.unwrap(),
                line.unwrap(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

#[cfg(feature = "logger")]
impl DebugConsole {
    pub(crate) fn log_init() {
        use log::LevelFilter;
        log::set_logger(&DebugConsole).unwrap();
        log::set_max_level(match option_env!("LOG") {
            Some("error") => LevelFilter::Error,
            Some("warn") => LevelFilter::Warn,
            Some("info") => LevelFilter::Info,
            Some("debug") => LevelFilter::Debug,
            Some("trace") => LevelFilter::Trace,
            _ => LevelFilter::Debug,
        });
    }
}

#[cfg(feature = "logger")]
ph_ctor!(
    CONSOLE_INIT,
    crate::ctor::CtorType::HALDriver,
    DebugConsole::log_init
);
