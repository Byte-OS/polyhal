#[cfg(not(feature = "vga_text"))]
mod com;
#[cfg(feature = "vga_text")]
mod vga_text;

pub fn init_early() {
    #[cfg(not(feature = "vga_text"))]
    com::init();
    #[cfg(feature = "vga_text")]
    vga_text::init();
}
