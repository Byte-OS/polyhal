use crate::debug::DebugConsole;

impl DebugConsole {
    pub fn putchar(c: u8) {
        print!("{}", c as char);
    }
}
