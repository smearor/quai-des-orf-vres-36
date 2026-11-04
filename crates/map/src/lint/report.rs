//! `LintReport` - the whole dataset quality verdict of
//! [`GameMap::lint`].

use crate::Transport;
use crate::lint::density::TransportDensity;
use crate::lint::finding::Finding;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

/// All findings plus informational density metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintReport {
    /// Quality violations, grouped per criterion in stable order.
    pub findings: Vec<Finding>,
    /// Per-transport density (edges, served stations, avg degree).
    pub density: BTreeMap<Transport, TransportDensity>,
}

impl LintReport {
    /// Whether the dataset satisfies every criterion.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}
