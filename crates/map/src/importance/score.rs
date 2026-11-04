//! `ImportanceScore` - a station's weighted strategic score.

/// A station's strategic importance score (weighted sum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportanceScore(pub u32);
