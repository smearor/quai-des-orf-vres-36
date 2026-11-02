//! The move domain: what a hop/move is, how it is requested, recorded
//! and rejected.

pub mod error;
pub mod hop;
pub mod kind;
pub mod record;
pub mod request;
pub mod shape;

pub use error::MoveError;
pub use hop::Hop;
pub use kind::MoveKind;
pub use record::MoveRecord;
pub use request::MoveRequest;
pub use shape::MoveShape;
