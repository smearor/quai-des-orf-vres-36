use qdo36_map::StationId;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::TicketCounts;
use qdo36_moves::GameLog;
use qdo36_moves::GameOutcome;
use qdo36_moves::GamePhase;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

/// The complete state of one game snapshot.
///
/// `fugitive_pos` is the hidden truth - it must never leak into public
/// projections (HUD, public MCP state); the private view is served
/// only inside the shield zone / to the Fugitive token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    /// Detective pawn positions.
    #[serde(with = "ordered_map")]
    pub positions: BTreeMap<PawnId, StationId>,
    /// Fugitive's true position - hidden from the public view.
    pub fugitive_pos: StationId,
    /// Ticket inventories per role.
    #[serde(with = "ordered_map")]
    pub tickets: BTreeMap<Role, TicketCounts>,
    /// Current round.
    pub round: RoundNumber,
    /// Whose turn it is.
    pub active: Role,
    /// Game lifecycle phase.
    pub phase: GamePhase,
    /// Committed move log.
    pub log: GameLog,
    /// Set once the game ends.
    pub outcome: Option<GameOutcome>,
}

/// Serializes a `BTreeMap` as an ordered sequence of key/value pairs
/// (`[[k, v], ...]`), so TOML/JSON writers that require string keys
/// still round-trip typed keys.
mod ordered_map {
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;
    use std::collections::BTreeMap;

    pub(super) fn serialize<K, V, S>(map: &BTreeMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Serialize + Ord,
        V: Serialize,
        S: Serializer,
    {
        map.iter().collect::<Vec<_>>().serialize(serializer)
    }

    pub(super) fn deserialize<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
    where
        K: Deserialize<'de> + Ord,
        V: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Vec::<(K, V)>::deserialize(deserializer).map(|pairs| pairs.into_iter().collect())
    }
}
