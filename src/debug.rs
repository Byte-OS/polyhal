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
pub struct DebugConsole;

// Write string through DebugConsole
impl Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().into_iter().for_each(|x| Self::putchar(*x));
        Ok(())
    }
}
