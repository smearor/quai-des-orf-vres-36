//! `ParallelShare` - one lane in the parallel-transports view.

use qdo36_map::LineName;
use qdo36_map::Transport;

/// One lane a shared hop draws in the parallel-transports view:
/// a whole transport, or - with the metro overlay on - a single
/// metro line so every line keeps its own lane and line color.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParallelShare {
    /// A whole transport mode (taxi, bus; also metro while the
    /// metro overlay is off and it draws one generic color).
    Transport(Transport),
    /// A single metro line (metro overlay on).
    MetroLine(LineName),
}

impl ParallelShare {
    /// The share a metro edge takes in this view: per line when the
    /// metro overlay runs, the whole transport otherwise.
    pub fn for_metro(line: Option<&LineName>, metro_overlay: bool) -> Self {
        match (metro_overlay, line) {
            (true, Some(line)) => Self::MetroLine(line.clone()),
            _ => Self::Transport(Transport::Metro),
        }
    }
}
