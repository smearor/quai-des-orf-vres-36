//! Geometric shell types: point sequences in map mm, tagged by
//! whether the line closes back on itself.

pub mod line;
pub mod polygon;
pub mod polyline;

pub(crate) use line::Line;
pub use polygon::Polygon;
pub use polyline::Polyline;
