//! `Sharing` - the lanes sharing each hop's corridor.

use crate::ui::map_scene::lane::LaneSide;
use qdo36_map::StationPair;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

/// The lanes sharing each hop as a map of ordered lane groups -
/// Derefs to the `HashMap`.
#[derive(Debug, Clone, Default)]
pub struct Sharing<C>(pub HashMap<StationPair, C>);

impl<C> Sharing<C> {
    /// `want`'s centered rank among the lanes serving `hop` - 0
    /// when the hop or `want` is not served.
    pub fn rank<S>(&self, hop: &StationPair, want: &S) -> f64
    where
        for<'a> &'a C: IntoIterator<Item = &'a S>,
        S: PartialEq,
    {
        let Some(shares) = self.0.get(hop) else { return 0.0 };
        let count = shares.into_iter().count();
        shares
            .into_iter()
            .position(|share| share == want)
            .map(|rank| rank as f64 - (count as f64 - 1.0) / 2.0)
            .unwrap_or(0.0)
    }

    /// `want`'s centered rank on each of `hops`' lanes.
    pub fn ranks<S>(&self, hops: &[StationPair], want: &S) -> Vec<f64>
    where
        for<'a> &'a C: IntoIterator<Item = &'a S>,
        S: PartialEq,
    {
        hops.iter().map(|key| self.rank(key, want)).collect()
    }

    /// The lane offset `want` draws at on `hop`: centered rank ×
    /// `spacing` × canonical direction × resolved `side`.
    pub fn offset<S>(&self, hop: &StationPair, want: &S, spacing: f64, dir: f64, side: LaneSide) -> f64
    where
        for<'a> &'a C: IntoIterator<Item = &'a S>,
        S: PartialEq,
    {
        self.rank(hop, want) * spacing * dir * side.factor()
    }
}

impl<C> Deref for Sharing<C> {
    type Target = HashMap<StationPair, C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for Sharing<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
