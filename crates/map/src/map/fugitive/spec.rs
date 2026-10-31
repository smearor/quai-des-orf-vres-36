use crate::StationId;
use crate::map::FugitiveAlias;
use serde::Deserialize;
use serde::Serialize;

/// The city's fugitive: alias plus home station.
///
/// Mirrors [`crate::AnchorSpec`]: Berlin's Dagobert (Arno Funke, the
/// KaDeWe extortionist - his home station is Wittenbergplatz), Paris
/// has Monsieur Corbeau beneath the Eiffel tower. The home station
/// can seed the fugitive's hidden start, flavor markers and story
/// beats.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FugitiveSpec {
    /// Display alias ("Monsieur Corbeau", "Dagobert").
    pub alias: FugitiveAlias,
    /// The fugitive's home station (thematic turf, validated to
    /// exist).
    pub station: StationId,
}
