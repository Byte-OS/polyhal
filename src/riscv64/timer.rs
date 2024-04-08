use crate::currrent_arch::boards::CLOCK_FREQ;
use crate::set_timer;
use crate::time::Time;
use riscv::register::{sie, time};

impl Time {
    #[inline]
    pub fn get_freq() -> usize {
        CLOCK_FREQ
    }

    #[inline]
    pub fn now() -> Self {
        Self(time::read())
    }
}

// 设置下一次时钟中断触发时间
#[inline]
pub fn set_next_timeout() {
    // 调用sbi设置定时器
    set_timer(time::read() + CLOCK_FREQ / 100);
}

pub fn init() {
    unsafe {
        sie::set_stimer();
    }
    set_next_timeout();
    info!("initialize timer interrupt");
}
