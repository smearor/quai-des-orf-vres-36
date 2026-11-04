//! `MeanDegree` - mean edge-degree across a layer's served stations.

use crate::EdgeCount;
use crate::StationCount;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Mean edge-degree of a served station on one transport layer.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MeanDegree(f64);

impl MeanDegree {
    /// `2 * edges / served` - every edge contributes two stubs.
    pub fn of(edges: EdgeCount, served: StationCount) -> Self {
        Self(if served.get() > 0 {
            2.0 * f64::from(edges.get()) / f64::from(served.get())
        } else {
            0.0
        })
    }

    /// The bare number, for arithmetic and comparisons.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for MeanDegree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}
