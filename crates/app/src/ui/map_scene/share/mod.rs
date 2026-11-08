//! `share` - who claims a drawn lane slot on a hop shared by
//! several edges: `HopShare` in the standard view, `ParallelShare`
//! in the parallel-transports view.

pub mod hop;
pub mod parallel;

pub use hop::HopShare;
pub use parallel::ParallelShare;
