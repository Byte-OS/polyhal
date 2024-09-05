mod com;
#[cfg(feature = "graphic")]
mod font;
#[cfg(feature = "graphic")]
mod graphic;
#[cfg(feature = "graphic")]
mod vga_text;

#[cfg(feature = "graphic")]
mod keyboard;

pub(crate) use com::init as init_com;

#[cfg(feature = "graphic")]
pub(crate) use graphic::init as init_fb;

#[cfg(feature = "graphic")]
pub(crate) use vga_text::init as init_vga;

use super::DebugConsole;

impl DebugConsole {
    #[inline]
    pub fn putchar(c: u8) {
        if c == b'\n' {
            com::putchar(b'\r');
            #[cfg(feature = "graphic")]
            graphic::putchar(b'\r');
        }
        com::putchar(c);

        #[cfg(feature = "graphic")]
        match graphic::is_graphic() {
            true => graphic::putchar(c),
            false => vga_text::putchar(c)
        }
    }

    #[inline]
    pub fn getchar() -> Option<u8> {
        #[cfg(not(feature = "graphic"))]
        {
            com::getchar()
        }
        #[cfg(feature = "graphic")]
        {
            keyboard::get_key()
        }
    }

    /// Set the color of the current state.
    #[inline]
    #[cfg(feature = "graphic")]
    pub(crate) fn set_color(color: u32) {
        graphic::set_color(color)
    }
}
