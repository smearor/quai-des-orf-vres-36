//! `route` - the routes the renderer draws: `RoutePlan` (an
//! edge's drawn route plus its lane offsets) and `WaterRoute`
//! (a boat route along a water course plus its entry arc).

pub mod plan;
pub mod water;

pub use plan::RoutePlan;
pub use water::WaterRoute;
