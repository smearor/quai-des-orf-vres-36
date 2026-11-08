//! `Steps` - a subdivision count of an approximated arc.

/// How many straight segments approximate a curved shape (the
/// flatiron nose arc) - a positive count, never zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Steps(usize);

impl Steps {
    /// A subdivision count.
    pub const fn new(count: usize) -> Self {
        Self(count)
    }

    /// The count for range loops.
    pub fn as_usize(self) -> usize {
        self.0
    }

    /// The count as `f64` for interpolation factors.
    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}
