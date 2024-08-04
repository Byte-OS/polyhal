#[cfg(not(any(feature = "vga_text", feature = "graphic")))]
mod com;
#[cfg(feature = "vga_text")]
mod vga_text;
#[cfg(feature = "graphic")]
mod font;
#[cfg(feature = "graphic")]
mod graphic;
mod keyboard;

#[cfg(not(any(feature = "vga_text", feature = "graphic")))]
pub(crate) use com::init as init_early;

#[cfg(feature = "vga_text")]
pub(crate) use vga_text::init as init_early;

#[cfg(feature = "graphic")]
pub(crate) use graphic::init as init_early;
