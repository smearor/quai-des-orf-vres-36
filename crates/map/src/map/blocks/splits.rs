//! `Splits` - a segment's cut parameters along 0..1.

/// A segment's cut parameters along 0..1: where crossings and
/// snapped endpoints split it into atomic fragments.
#[derive(Clone, Default)]
pub(super) struct Splits(pub Vec<f64>);

impl Splits {
    /// Adds a cut at parameter `t`.
    pub fn push(&mut self, t: f64) {
        self.0.push(t);
    }

    /// Closes the range: adds the 0/1 endpoints, sorts, dedups -
    /// after `finish` the parameters tile the segment.
    pub fn finish(&mut self) {
        self.0.push(0.0);
        self.0.push(1.0);
        self.0.sort_by(f64::total_cmp);
        self.0.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    }

    /// The consecutive cut windows (after [`Self::finish`]).
    pub fn cuts(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.0.windows(2).map(|w| (w[0], w[1]))
    }
}
