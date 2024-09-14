 /// Call SBI_SHUTDOWN to close the machine. Exit qemu if you are using qemu.
 #[inline]
 pub fn shutdown() -> ! {
     // sbi_rt::legacy::shutdown();
     sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
     unreachable!()
 }
 