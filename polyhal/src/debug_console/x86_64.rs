mod com;
#[cfg(feature = "graphic")]
mod font;
#[cfg(feature = "graphic")]
mod graphic;
mod vga_text;

#[cfg(feature = "graphic")]
mod keyboard;

#[cfg(feature = "graphic")]
pub use graphic::init as init_fb;

use crate::ctor::CtorType;

use super::DebugConsole;

impl DebugConsole {
    #[inline]
    pub fn putchar(c: u8) {
        if c == b'\n' {
            com::putchar(b'\r');
            vga_text::putchar(c);
            #[cfg(feature = "graphic")]
            graphic::putchar(b'\r');
        }
        com::putchar(c);
        vga_text::putchar(c);

        #[cfg(feature = "graphic")]
        graphic::putchar(c);
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

ph_ctor!(X86_INIT_COM, CtorType::Primary, || {
    com::init();
    vga_text::init();
});
