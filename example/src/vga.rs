use core::fmt::{Error, Write};

use polyhal::utils::MutexNoIrq;

static VGA_BUFFER: MutexNoIrq<VGAPos> = MutexNoIrq::new(VGAPos::new());

#[repr(C)]
#[derive(Default)]
pub struct FChar(u8, u8);

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
}

pub struct TestWrite;

impl Write for TestWrite {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        vga_print(s);
        Ok(())
    }
}

pub fn vga_putchar(c: u8) {
    VGA_BUFFER.lock().putchar(c);
}

pub fn vga_print(s: &str) {
    let mut vga_buffer = VGA_BUFFER.lock();
    s.as_bytes().iter().for_each(|c| vga_buffer.putchar(*c));
}

pub fn main_func() {
    VGA_BUFFER.lock().clear();
    for i in 0..27 {
        TestWrite.write_fmt(format_args!("This is line {i}\n\r"));
    }
    TestWrite.write_str("This is a toooooooooooooooo lonnnnnnnnnnnnnnnnnnnnnnnnnnng print to test the new line function.\n");
    TestWrite.write_str(
        "This is a toooooooooooooooo lonnnnnnnnnnnnnnnnnnnnnnnnnnng print to test the new line\n",
    );

    // for i in (0xa0000..0xc0000).step_by(1) {
    //     unsafe {
    //         *(i as *mut u8) = 0xff;
    //     }
    // }

    // TestWrite.write_str("errorerror\rThis is to test \\r function");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
