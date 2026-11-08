//! `scene` - the scene's render inputs: `SceneLayers` (which
//! sections draw, `TransportLayers` for the per-transport flags),
//! `RenderParams` (global display factors) and `SceneState`
//! (everything drawn in one pass).

pub mod layers;
pub mod params;
pub mod state;
pub mod transports;

pub use layers::SceneLayers;
pub use params::RenderParams;
pub use state::SceneState;
pub use transports::TransportLayers;
