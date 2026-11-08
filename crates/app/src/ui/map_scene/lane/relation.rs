//! `SideRelation` - how a lane's side carries through a vertex.

/// The side a lane takes through a vertex relative to the
/// previous hop's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideRelation {
    /// Keeps its spatial side (`side(to) = side(from)`).
    Same,
    /// Flips to the mirrored side (`side(to) = -side(from)`).
    Flip,
}

impl SideRelation {
    /// The relation for a signed product `c` (`< 0` flips).
    pub fn of(product: f64) -> Self {
        if product < 0.0 { Self::Flip } else { Self::Same }
    }
}
