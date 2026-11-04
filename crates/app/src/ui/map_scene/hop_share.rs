//! Who claims a slot on a route hop shared by several drawn edges.

use qdo36_map::LineName;

/// Who claims a slot on a hop shared by multiple drawn routes:
/// metro edges share per distinct line (same line, same slot),
/// underground tunnels share per edge (tunnels have no lines).
#[derive(Clone, PartialEq)]
pub enum HopShare {
    /// A metro edge claims the hop under its line name, so two
    /// edges of the same line share one slot instead of doubling.
    Metro(LineName),
    /// A line-less edge (underground tunnel) claims its own slot
    /// per edge index.
    Edge(usize),
}
