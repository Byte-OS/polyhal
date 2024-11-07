use core::sync::atomic::{AtomicUsize, Ordering};

/// Structure Store usize and data like this
///
/// Initialize in the Boot Stage. Only initialize once.
pub(crate) struct InitNum(AtomicUsize);

impl InitNum {
    /// Create a new InitNum
    pub const fn new(num: usize) -> InitNum {
        Self(AtomicUsize::new(0))
    }

    /// Get the number of the elements
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.0.load(Ordering::SeqCst) != 0
    }

    /// Initialize if not already initialized
    #[inline]
    pub fn init(&self, value: usize) {
        let _ = self
            .0
            .compare_exchange(0, value, Ordering::Acquire, Ordering::Relaxed);
    }

    /// Get the number in the InitNum structure
    #[inline]
    pub fn get(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    /// Get the Some(data) if has been initialized, return None otherwise.
    #[inline]
    pub fn get_option(&self) -> Option<usize> {
        let data = self.0.load(Ordering::SeqCst);
        match data {
            0 => None,
            _ => Some(data),
        }
    }
}
