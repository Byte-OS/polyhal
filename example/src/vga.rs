
pub fn main_func() {
    // TestWrite.write_str("errorerror\rThis is to test \\r function");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
