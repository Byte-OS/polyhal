use x86::io::{inb, outb};

use crate::{components::debug_console::{println, DebugConsole}, utils::MutexNoIrq};

// 键盘扫描码到 ASCII 字符的映射表
const SCAN_CODE_TO_ASCII: [u8; 58] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'+', 08,
    b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 10, 0,
    b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\',
    b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 0, b'*', 0,  b' '
];

static VGA_BUFFER: MutexNoIrq<VGAPos> = MutexNoIrq::new(VGAPos::new());

#[repr(C)]
pub struct FChar(u8, u8);

impl FChar {
    const fn default() -> Self {
        Self(0, 7)
    }
}

pub struct VGAPos {
    x: usize,
    y: usize,
}

impl VGAPos {
    /// How many characters in one row.
    const ROW_C_MAX: usize = 80;
    /// How many rows in the screen.
    const COL_C_MAX: usize = 25;
    /// Buffer pointer of the vga buffer.
    const VGA_BUFFER_PTR: *mut FChar = 0xb8000 as *mut FChar;

    /// Create a new VGA Buffer include the position information.
    pub const fn new() -> Self {
        VGAPos { x: 0, y: 0 }
    }

    /// Put a character to the vga buffer.
    pub fn putchar(&mut self, c: u8) {
        let vga_buffer = Self::VGA_BUFFER_PTR as *mut FChar;
        match c {
            b'\n' => {
                self.x = 0;
                self.y += 1;
            }
            b'\r' => {
                self.x = 0;
            }
            _ => {
                unsafe {
                    vga_buffer.add(self.offset()).write_volatile(FChar(c, 0x7));
                }
                self.x += 1;
            }
        }
        if self.x >= Self::ROW_C_MAX {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= Self::COL_C_MAX {
            self.scroll_up(1);
        }
        self.move_cursor();
    }

    /// Scroll the screen to the specified position.
    pub fn scroll_up(&mut self, line: usize) {
        let vga_buffer = Self::VGA_BUFFER_PTR;
        for i in 0..Self::pos_offset(self.x, self.y - line) {
            unsafe {
                vga_buffer
                    .add(i)
                    .write_volatile(vga_buffer.add(i + line * Self::ROW_C_MAX).read_volatile());
            }
        }
        for i in Self::pos_offset(self.x, self.y - line)..Self::pos_offset(self.x, self.y) {
            unsafe {
                vga_buffer.add(i).write_volatile(FChar::default());
            }
        }
        self.y -= line;
    }

    /// Clear the screen.
    pub fn clear(&self) {
        let vga_buffer = Self::VGA_BUFFER_PTR;
        for i in 0..(Self::COL_C_MAX * Self::ROW_C_MAX) {
            unsafe {
                vga_buffer.add(i).write_volatile(FChar::default());
            }
        }
    }

    /// Get the offset of the current position.
    pub const fn offset(&self) -> usize {
        Self::pos_offset(self.x, self.y)
    }

    /// Get the position offset of the current position.
    pub const fn pos_offset(x: usize, y: usize) -> usize {
        y * Self::ROW_C_MAX + x
    }

    /// Move the cursor to the specified position.
    pub fn move_cursor(&self) {
        let position = self.offset();
        unsafe {
            outb(0x3d4, 0x0f);
            outb(0x3d5, (position & 0xff) as u8);
            outb(0x3d4, 0x0e);
            outb(0x3d5, (position >> 8) as u8);
        }
    }
}

/// Implement for debug console.
impl DebugConsole {
    pub fn putchar(c: u8) {
        VGA_BUFFER.lock().putchar(c);
    }

    pub fn getchar() -> Option<u8> {
        let c = unsafe {
            inb(0x60)
        };
        SCAN_CODE_TO_ASCII.get(c as usize).cloned()
    }
}

/// Init VBE Text Mode configuration
pub(crate) fn init() {
    let cursor_start = 14;
    let cursor_end = 15;

    unsafe {
        outb(0x3D4, 0x0A);
        outb(0x3D5, (inb(0x3D5) & 0xC0) | cursor_start);

        outb(0x3D4, 0x0B);
        outb(0x3D5, (inb(0x3D5) & 0xE0) | cursor_end);
    }
    VGA_BUFFER.lock().clear();
}
