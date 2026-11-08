//! `decoration` - decorative streets inside a block.
//!
//! Visual richness for blocks big enough to hold it: decorative
//! streets are chords crossing the block interior (they do not
//! split the block - they just carve a corridor the houses stay
//! out of).

pub mod street;
pub mod streets;

pub use street::Street;
pub use streets::Streets;
