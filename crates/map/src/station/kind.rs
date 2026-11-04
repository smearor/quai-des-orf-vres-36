use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Special role a station can play, in addition to being a regular
/// node of the graph.
///
/// Stored per station as a `BTreeSet` - a river stop can also be a
/// start station (e.g. the Eiffel tower pier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StationKind {
    /// Eligible in the random start deal.
    Start,
    /// Batobus pier: `Transport::Batobus` edges may terminate here.
    Batobus,
    /// Underground access hatch: `Transport::Underground` edges may
    /// terminate here (Paris: Égouts, Berlin: Unterwelten).
    Underground,
    /// Bridge over a water course: `taxi`/`bus` edges may only cross
    /// a `backdrop.water` polyline when at least one endpoint carries
    /// this kind. Every `batobus` pier must also be a bridge - the
    /// boat docks at the bridge landing.
    Bridge,
}

impl fmt::Display for StationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Start => "start",
            Self::Batobus => "batobus",
            Self::Underground => "underground",
            Self::Bridge => "bridge",
        };
        f.write_str(name)
    }
}
