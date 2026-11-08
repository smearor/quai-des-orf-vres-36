//! City blocks: the free areas the map is cut into by its surface
//! street mesh (taxi/bus edges), the river courses and the map
//! border. Metro and underground edges run below the surface and do
//! not cut the map into blocks.

pub mod angle;
pub mod buildings;
pub mod cut;
pub mod cut_segment;
pub mod decoration;
pub mod depth;
pub mod extract;
pub mod face;
pub mod faces;
pub mod ground;
pub mod height;
pub mod house;
pub mod lane_map;
pub mod lanes;
pub mod layout;
pub mod noise;
pub mod obstacles;
pub mod splits;
pub mod steps;
pub mod units;
pub mod variant;

pub use angle::Angle;
pub use buildings::BuildingParams;
pub use cut::BlockCut;
pub use cut_segment::CutSegment;
pub use decoration::Street;
pub use depth::Depth;
pub use face::BlockFace;
pub use faces::BlockFaces;
pub use ground::Ground;
pub use height::Height;
pub use house::House;
pub use lane_map::LaneMap;
pub use lanes::HopLanes;
pub use layout::BlockLayout;
pub use obstacles::Obstacles;
pub use steps::Steps;
pub use tangible_objects_model::SquareMillimeters;
pub use units::Units;
pub use variant::Variant;
