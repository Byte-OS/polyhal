use crate::{arch::consts::VIRT_ADDR_START, utils::MutexNoIrq};

use super::font::BIT_FONTS;

/// TIPS: This should always be a multiple of 2, or 1, But not 0.
const SCALE: usize = 1;

pub struct GraphicConsole {
    x: usize,
    y: usize,
    /// The pixels in the row.
    width: usize,
    /// The number of rows.
    height: usize,
    /// The bytes of a line.
    pitch: usize,
    /// Frame buuffer pointer.
    ptr: usize,
    /// The font color.
    color: u32,
    /// Is ANSI Control mode.
    control: bool,
    step: usize,
    value: usize,
}

impl GraphicConsole {
    /// The width of the font in the console.
    const F_WIDTH: usize = 8 * SCALE;
    /// The height of the font in the console.
    const F_HEIGHT: usize = 16 * SCALE;

    /// Create a new graphic console.
    const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            ptr: 0xfd000000,
            width: 1366,
            height: 768,
            pitch: 1366,
            color: 0xffffffff,
            control: false,
            step: 0,
            value: 0,
        }
    }

    /// Scroll the screen to the specified position.
    fn scroll_up(&mut self, line: usize) {
        assert_eq!(self.x, 0);
        let ptr = self.ptr as *mut u64;
        let offset = line * Self::F_HEIGHT * self.pitch / 8;
        let blank_offset = (self.y - line) * self.pitch / 8;
        unsafe {
            ptr.copy_from_nonoverlapping(ptr.add(offset), blank_offset);
            core::slice::from_raw_parts_mut(ptr.add(blank_offset), offset).fill(0);
        }
        self.y -= line * Self::F_HEIGHT;
    }

    /// Put a character to the Screen.
    fn put_char(&mut self, c: u8) {
        // Check the ansi settings.
        if self.control {
            match self.step {
                0 => {
                    if c == b'[' {
                        self.step += 1;
                        return;
                    } else {
                        self.control = false;
                    }
                }
                1 => {
                    match c {
                        b'0'..=b'9' => {
                            self.step = 0;
                            self.control = false;
                            self.scroll_up(self.value as usize);
                            self.y = (self.y / Self::F_HEIGHT) * Self::F_HEIGHT;
                        }
                        b'J' => {
                            self.control = false;
                            return;
                        }
                        b'm' => {
                            match self.value {
                                0 => self.color = 0xffffff,
                                32 => self.color = 0x00ff00,
                                34 => self.color = 0x33ccff,
                                _ => {}
                            }
                            self.control = false;
                            return;
                        }
                        _ => self.control = false,
                    }
                    // if c >= b'0' && c <= b'9' {
                    //     self.value = (self.value * 10) + (c - b'0') as usize;
                    //     return;
                    // } else if c == b'J' {
                    //     self.control = false;
                    //     return;
                    // } else if c == b'm' {
                    //     match self.value {
                    //         0 => self.color = 0xffffff,
                    //         32 => self.color = 0x00ff00,
                    //         34 => self.color = 0x33ccff,
                    //         _ => {}
                    //     }
                    //     self.control = false;
                    //     return;
                    // } else {
                    //     self.control = false;
                    // }
                }
                _ => self.control = false,
            }
        }

        match c {
            b'\n' => {
                self.y += Self::F_HEIGHT;
                self.x = 0;
            }
            b'\r' => self.x = 0,
            b'\t' => self.x = Self::F_WIDTH * 4,
            // Backspace character
            0x08 => self.backspace(),
            // ANSI Control Mode
            0x1b => {
                self.control = true;
                self.step = 0;
                self.value = 0;
            }
            _ => {
                let bit_offset = match c as usize * 0x10 < BIT_FONTS.len() {
                    true => c as usize * 0x10,
                    _ => 0,
                };

                for y in 0..16 {
                    let word = BIT_FONTS[bit_offset + y];

                    for x in 0..8 {
                        let color = match word & (1 << (7 - x)) != 0 {
                            true => self.color,
                            false => 0,
                        };

                        self.write_pixel(y, x, color);
                    }
                }

                self.x += Self::F_WIDTH;
            }
        }

        // If the last space is not enough for a character, then newline.
        if self.x > self.width - Self::F_WIDTH {
            self.x = 0;
            self.y += Self::F_HEIGHT;
        }
        // If the last line is not enough for a character, scroll up 1 line.
        if self.y > self.height - Self::F_HEIGHT {
            self.scroll_up(1);
        }
    }

    /// Backspace character.
    fn backspace(&mut self) {
        // println!("backspace character");
        if self.x > Self::F_WIDTH {
            self.x -= Self::F_WIDTH;

            let ptr = self.current_ptr();
            for y in 0..16 {
                for x in 0..8 {
                    unsafe {
                        ptr.add(self.line_offset(y, x)).write_volatile(0);
                    }
                }
            }
        }
    }

    /// Clear the screen.
    #[inline]
    fn clear(&self) {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr as *mut u64, self.height * self.pitch / 4 / 2)
                .fill(0);
        }
    }

    /// Write a Pixel to graphic frame buffer.
    #[inline]
    fn write_pixel(&self, y: usize, x: usize, color: u32) {
        unsafe {
            self.current_ptr()
                .add(self.line_offset(y, x))
                .write_volatile(color);
        }
    }

    /// Get the current pointer of the current.
    const fn current_ptr(&self) -> *mut u32 {
        (self.ptr + self.y * self.pitch + self.x * 4) as *mut u32
    }

    /// Get the offset of the given line number.
    const fn line_offset(&self, line: usize, x: usize) -> usize {
        line * self.pitch / 4 + x
    }
}

static GRAPHIC_CONSOLE: MutexNoIrq<GraphicConsole> = MutexNoIrq::new(GraphicConsole::new());

#[inline]
pub(super) fn putchar(c: u8) {
    GRAPHIC_CONSOLE.lock().put_char(c);
}

/// Set the color of the current state.
#[inline]
pub(super) fn set_color(color: u32) {
    GRAPHIC_CONSOLE.lock().color = color;
}

/// Init the graphics console's information, includes frame buffer addresse, width and height.
pub fn init(addr: usize, width: usize, height: usize, pitch: usize) {
    let mut g_console = GRAPHIC_CONSOLE.lock();
    g_console.ptr = addr | VIRT_ADDR_START;
    g_console.width = width;
    g_console.height = height;
    g_console.pitch = pitch;
    g_console.clear();
    drop(g_console);
}
