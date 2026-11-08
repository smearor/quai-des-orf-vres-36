//! `SceneLayers` - which scene sections the renderer draws.
//!
//! Toggled live via the MCP `set_layers` command for visual
//! debugging; defaults to everything on.

use crate::ui::map_scene::LabelMode;
use crate::ui::map_scene::LayerName;
use crate::ui::map_scene::TransportLayers;
use crate::ui::style::HouseMode;
use qdo36_map::LineName;
use qdo36_map::Transport;
use std::collections::BTreeSet;

/// Which scene sections render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneLayers {
    /// Water polylines (the Seine).
    pub water: bool,
    /// City-block outlines: the free areas enclosed by streets,
    /// river and border.
    pub blocks: bool,
    /// Backdrop landmark pins.
    pub landmarks: bool,
    /// Per-transport edge visibility.
    pub transports: TransportLayers,
    /// Transports drawn in their own colors; the rest render in the
    /// neutral `edge` ink (standard view).
    pub highlight: BTreeSet<Transport>,
    /// Metro lines hidden by name (e.g. `"M4"`) - on top of the
    /// `metro` transport flag.
    pub hidden_lines: BTreeSet<LineName>,
    /// Taxi/bus hops shared by several transports draw side by
    /// side in each transport's color (metro-line style).
    pub parallel_transports: bool,
    /// Station discs.
    pub stations: bool,
    /// Station name labels.
    pub labels: bool,
    /// Station number labels inside the disc.
    pub numbers: bool,
    /// Strategic-importance score labels beside stations.
    pub scores: bool,
    /// Label orientation for tabletop play.
    pub label_mode: LabelMode,
    /// Landmark artwork orientation: upright pins (`Screen`) or
    /// pins facing the nearest table edge (`Radial`); each pin
    /// additionally dodges stations and edges.
    pub landmark_mode: LabelMode,
    /// District caption orientation (`Radial` default like the
    /// station labels).
    pub district_mode: LabelMode,
    /// Park caption orientation (`Radial` default like the
    /// station labels).
    pub park_mode: LabelMode,
    /// House rendering mode (`RadialIso` default: each block's
    /// houses extrude toward its nearest table edge).
    pub house_mode: HouseMode,
    /// The compass rose.
    pub compass: bool,
    /// Bridge-station markers (dataset debugging, MCP-only layer).
    pub bridges: bool,
}

impl SceneLayers {
    /// The named layer's visibility flag.
    pub fn get(&self, name: LayerName) -> bool {
        match name {
            LayerName::Taxi => self.transports.taxi,
            LayerName::Bus => self.transports.bus,
            LayerName::Metro => self.transports.metro,
            LayerName::MetroOverlay => self.highlight.contains(&Transport::Metro),
            LayerName::Underground => self.transports.underground,
            LayerName::Batobus => self.transports.batobus,
            LayerName::ParallelTransports => self.parallel_transports,
            LayerName::Water => self.water,
            LayerName::Blocks => self.blocks,
            LayerName::Landmarks => self.landmarks,
            LayerName::Stations => self.stations,
            LayerName::Labels => self.labels,
            LayerName::Numbers => self.numbers,
            LayerName::Scores => self.scores,
            LayerName::Compass => self.compass,
            LayerName::Bridges => self.bridges,
        }
    }

    /// Sets the named layer's visibility flag.
    pub fn set(&mut self, name: LayerName, on: bool) {
        if name == LayerName::MetroOverlay {
            if on {
                self.highlight.insert(Transport::Metro);
            } else {
                self.highlight.remove(&Transport::Metro);
            }
            return;
        }
        if let Some(flag) = self.flag_mut(name) {
            *flag = on;
        }
    }

    /// The bool flag a layer name maps to - `None` for layers
    /// that are not a plain flag (the metro overlay highlight).
    fn flag_mut(&mut self, name: LayerName) -> Option<&mut bool> {
        let flag = match name {
            LayerName::Taxi => &mut self.transports.taxi,
            LayerName::Bus => &mut self.transports.bus,
            LayerName::Metro => &mut self.transports.metro,
            LayerName::Underground => &mut self.transports.underground,
            LayerName::Batobus => &mut self.transports.batobus,
            LayerName::ParallelTransports => &mut self.parallel_transports,
            LayerName::Water => &mut self.water,
            LayerName::Blocks => &mut self.blocks,
            LayerName::Landmarks => &mut self.landmarks,
            LayerName::Stations => &mut self.stations,
            LayerName::Labels => &mut self.labels,
            LayerName::Numbers => &mut self.numbers,
            LayerName::Scores => &mut self.scores,
            LayerName::Compass => &mut self.compass,
            LayerName::Bridges => &mut self.bridges,
            LayerName::MetroOverlay => return None,
        };
        Some(flag)
    }
}

impl Default for SceneLayers {
    fn default() -> Self {
        Self {
            water: true,
            blocks: true,
            landmarks: true,
            transports: TransportLayers::default(),
            highlight: BTreeSet::new(),
            hidden_lines: BTreeSet::new(),
            parallel_transports: true,
            stations: true,
            labels: true,
            numbers: true,
            scores: false,
            label_mode: LabelMode::default(),
            landmark_mode: LabelMode::Screen,
            district_mode: LabelMode::default(),
            park_mode: LabelMode::default(),
            house_mode: HouseMode::default(),
            compass: false,
            bridges: false,
        }
    }
}
