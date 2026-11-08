//! City-block drawing: insets the cached block faces by the
//! corridor width of their bounding features, which depends on the
//! view mode - the parallel-transports overlay and the metro
//! overlay widen the street corridors.

pub mod geometry;
pub mod shapes;

pub use geometry::BlocksGeometry;
pub use geometry::blocks_key;
pub use shapes::BlockShapes;
