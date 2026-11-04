//! `ImportanceClass` - the station's strategic group (tertile).

/// The station's strategic importance, grouped for presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportanceClass {
    /// Low reach and few options - a quiet corner.
    Minor,
    /// Ordinary connectivity.
    Typical,
    /// High reach and many options - a transfer hub.
    Hub,
}
