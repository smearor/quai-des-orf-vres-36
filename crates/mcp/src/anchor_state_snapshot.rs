//! `AnchorStateSnapshot` - the live anchor surface state, serialized
//! for MCP consumers (WP05 data model).

use qdo36_tangibles::AnchorState;
use qdo36_tangibles::SurfaceMm;
use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

use crate::rotation_deg::RotationDeg;

/// The published anchor snapshot - `Option` fields are `None` while
/// the tower is not detected.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnchorStateSnapshot {
    /// Whether the Eiffel tower tangible is currently tracked.
    pub detected: bool,
    /// Tangible centroid in device millimeters (tracked only).
    pub center_mm: Option<Vec2<Millimeters>>,
    /// Tangible orientation in degrees (tracked only).
    pub rotation_deg: Option<RotationDeg>,
    /// Touch surface size in millimeters.
    pub screen_mm: Option<SurfaceMm>,
}

impl AnchorStateSnapshot {
    /// Captures the snapshot from the app's anchor state and the
    /// connected surface.
    pub fn captured(anchor: &AnchorState, surface: Option<&SurfaceMm>) -> Self {
        let tracked = matches!(anchor, AnchorState::Tracked(_));
        let pose = anchor.pose();
        Self {
            detected: tracked,
            center_mm: tracked.then_some(pose.center),
            rotation_deg: tracked.then(|| pose.rotation_rad.into()),
            screen_mm: surface.copied(),
        }
    }
}
