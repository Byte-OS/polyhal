/// Time struct and its interface
///
/// You can use this to get time from ticks
///
/// ### get current time
/// ```rust
/// Time::now();
/// ```
///
/// ### get current cpu's frequency
/// ```rust
/// Time::get_freq();
/// ```
///
/// ### get how many nanoseconds have passed.
/// ```rust
/// Time::now().to_nsec();
/// ```
///
/// ### get how many microseconds have passed.
/// ```rust
/// Time::now().to_usec();
/// ```
///
/// ### get how many millisecond have passed.
/// ```rust
/// Time::now().to_msec();
/// ```
///
/// ### get how may ticks have passed
/// ```rust
/// Time::now().raw();
/// ```
///
/// ### convert ticks to time
/// ```rust
/// Time::from_raw(Time::now().raw());
/// ```
///

#[derive(Clone, Copy, Debug)]
pub struct Time(pub(crate) usize);

impl Time {
    #[inline]
    pub fn to_msec(&self) -> usize {
        self.0 * 1_000 / Self::get_freq()
    }

    #[inline]
    pub fn to_usec(&self) -> usize {
        self.0 * 1_000_000 / Self::get_freq()
    }

    /// Converts hardware ticks to nanoseconds.
    #[inline]
    pub fn to_nsec(&self) -> usize {
        let freq = Self::get_freq();
        (self.0 / freq * 1_000_000_000) + (self.0 % freq) * 1_000_000_000 / freq
    }

    #[inline]
    pub const fn raw(&self) -> usize {
        self.0
    }

    #[inline]
    pub const fn new(raw: usize) -> Self {
        Self(raw)
    }
}

#[polyhal_macro::percpu]
static TEST_MACRO: usize = 1;
