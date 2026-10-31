use qdo36_material::DetectiveCountError;
use qdo36_material::PawnColor;
use qdo36_material::RoundNumber;
use thiserror::Error;

/// Errors of session setup and persistence.
#[derive(Debug, Error)]
pub enum SessionError {
    /// The requested detective count is out of range.
    #[error(transparent)]
    DetectiveCount(#[from] DetectiveCountError),
    /// Two seats share a pawn color.
    #[error("duplicate pawn color {0}")]
    DuplicatePawnColor(PawnColor),
    /// The color list does not match the detective count.
    #[error("{0} colors configured for {1} detectives")]
    ColorCountMismatch(usize, u8),
    /// The game length is zero.
    #[error("max rounds must be at least 1, got {0}")]
    InvalidMaxRounds(RoundNumber),
    /// A reveal round lies beyond the configured game length.
    #[error("reveal round {round} exceeds max_rounds {max_rounds}")]
    RevealBeyondMaxRounds {
        /// The offending reveal round.
        round: RoundNumber,
        /// The configured maximum.
        max_rounds: RoundNumber,
    },
    /// The autosave file could not be written.
    #[error("could not save game: {0}")]
    Save(String),
    /// The autosave file could not be loaded.
    #[error("could not load game: {0}")]
    Load(String),
}
