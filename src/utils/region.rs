use core::ops::Range;

#[derive(Debug, Clone)]
pub struct AddrRegion {
    pub start: usize,
    pub size: usize,
}

/// Implementation of `AddrRegion`
impl AddrRegion {
    /// Subtracts a region from the original region.
    pub(crate) fn sub_region(&mut self, mut other: AddrRegion) -> Option<AddrRegion> {
        // If the region was not overlapped.
        if self.start + self.size < other.start || other.start + other.size < self.start {
            return None;
        }
        // Optimize the memory region for calculating.
        if other.start < self.start {
            other.size -= self.start - other.start;
            other.start = self.start;
        }
        if other.start + other.size > self.start + self.size {
            other.size = self.start + self.size - other.start;
        }
        let (r1_start, r1_end) = (self.start, other.start);
        let (r2_start, r2_end) = (other.start + other.size, self.start + self.size);

        unimplemented!("SubRegion")
    }
}
