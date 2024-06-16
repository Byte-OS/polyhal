use core::fmt::Write;

/// This is a console for debugging,
/// If you want to use this logging
/// You need to use like this:
///
/// #### Put a char to output device(always uart)
/// ```rust
/// DebugConsole::putchar(b'3');
/// ```
///
/// ### Get a char from input device(always uart)
/// ```rust
/// DebugConsole::getchar();
/// ```

/// Print macro to print polyhal information with newline
pub(crate) macro println {
    () => {
        $crate::debug::print(format_args!("\n"))
    },
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::debug::print(format_args!("{}\n", format_args!($fmt $(, $($arg)+)?)))
    },
}

/// Display Platform Information with specified format
/// display_info!("item name", "{}", "format");
/// The output format like below:
/// item name             : format
pub(crate) macro display_info{
    () => {
        $crate::debug::print(format_args!("\n"))
    },
    ($item:literal,$fmt: expr $(, $($arg: tt)+)?) => {
        $crate::debug::print(format_args!("{:<26}: {}\n", $item, format_args!($fmt $(, $($arg)+)?)))
    }
}

/// Print the given arguments
#[inline]
pub(crate) fn print(args: core::fmt::Arguments) {
    DebugConsole.write_fmt(args).expect("can't print arguments");
}

pub struct DebugConsole;

// Write string through DebugConsole
impl Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().into_iter().for_each(|x| Self::putchar(*x));
        Ok(())
    }
}
