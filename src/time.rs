#[derive(Clone, Copy, Debug)]
pub struct Time(pub(crate) usize);

impl Time {
    /// Converts hardware ticks to nanoseconds.
    #[inline]
    pub fn to_msec(&self) -> usize {
        self.0 * 1_000 / Self::get_freq()
    }

    #[inline]
    pub fn to_usec(&self) -> usize {
        self.0 * 1000_000 / Self::get_freq()
    }

    #[inline]
    pub fn to_nsec(&self) -> usize {
        self.0 * 1000_000_000 / Self::get_freq()
    }

    #[inline]
    pub fn raw(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }
}
