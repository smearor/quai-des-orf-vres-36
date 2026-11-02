//! `SavedGame` - the persistent snapshot for autosave/resume.

use serde::Deserialize;
use serde::Serialize;

use crate::GameConfig;
use crate::GameSession;
use crate::GameState;
use crate::SessionError;

/// A complete session snapshot: config, current state and the
/// undo/redo stacks, so a restored game continues with full history.
///
/// Note: `state` carries Fugitive's hidden position - a save file is
/// private storage by definition and must be handled like the shield
/// zone, never as a public projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedGame {
    /// The configuration the game was started with.
    pub config: GameConfig,
    /// The live state at save time.
    pub state: GameState,
    /// Undo stack at save time.
    pub past: Vec<GameState>,
    /// Redo stack at save time.
    pub future: Vec<GameState>,
}

impl SavedGame {
    /// Captures a session for persistence.
    pub fn snapshot(session: &GameSession) -> Self {
        Self {
            config: session.config.clone(),
            state: session.state.clone(),
            past: session.past.clone(),
            future: session.future.clone(),
        }
    }

    /// Restores a session; the stored config is re-validated so a
    /// hand-edited or corrupt save cannot seed an invalid game.
    pub fn restore(self) -> Result<GameSession, SessionError> {
        self.config.validate()?;
        Ok(GameSession {
            config: self.config,
            state: self.state,
            past: self.past,
            future: self.future,
        })
    }

    /// Serializes to TOML.
    pub fn to_toml(&self) -> Result<String, SessionError> {
        toml::to_string_pretty(self).map_err(|err| SessionError::Save(err.to_string()))
    }

    /// Parses from TOML.
    pub fn from_toml(text: &str) -> Result<Self, SessionError> {
        toml::from_str(text).map_err(|err| SessionError::Load(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qdo36_map::{GameMap, Transport};
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use qdo36_material::TicketKind;
    use qdo36_moves::{MoveKind, MoveRequest};

    /// A session three moves into a game.
    fn played() -> GameSession {
        let map = GameMap::bundled().unwrap_or_else(|_| panic!("bundled map must load"));
        let mut session = GameSession::deal(GameConfig::for_map(map.meta()), &map, &mut StdRng::seed_from_u64(36)).unwrap_or_else(|_| panic!("deal"));
        session.begin().unwrap_or_else(|_| panic!("begin"));
        for _ in 0..3 {
            let hop = session
                .legal_destinations(&map, session.state.active)
                .first()
                .copied()
                .unwrap_or_else(|| panic!("a legal move"));
            let ticket = match hop.transport {
                Transport::Taxi => TicketKind::Taxi,
                Transport::Bus => TicketKind::Bus,
                Transport::Metro => TicketKind::Metro,
                Transport::Batobus | Transport::Underground => TicketKind::Black,
            };
            session
                .request_move(
                    &map,
                    &MoveRequest {
                        actor: session.state.active,
                        ticket,
                        kind: MoveKind::Single(hop),
                    },
                )
                .unwrap_or_else(|err| panic!("{err}"));
        }
        session
    }

    #[test]
    fn toml_roundtrip_restores_the_session() {
        let session = played();
        let saved = SavedGame::snapshot(&session);
        let toml = saved.to_toml().unwrap_or_else(|_| panic!("ser"));
        let parsed = SavedGame::from_toml(&toml).unwrap_or_else(|_| panic!("de"));
        let restored = parsed.restore().unwrap_or_else(|_| panic!("restore"));
        assert_eq!(restored.state, session.state, "state survives the roundtrip");
        assert_eq!(restored.past.len(), session.past.len(), "undo stack survives");
    }

    #[test]
    fn restore_rejects_a_corrupt_config() {
        let session = played();
        let mut saved = SavedGame::snapshot(&session);
        saved.config.max_rounds = qdo36_material::RoundNumber::new(0);
        assert!(saved.restore().is_err(), "invalid config must fail restore");
    }

    #[test]
    fn restore_rejects_corrupt_toml() {
        assert!(SavedGame::from_toml("not = [toml").is_err());
    }
}
