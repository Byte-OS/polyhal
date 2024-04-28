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
