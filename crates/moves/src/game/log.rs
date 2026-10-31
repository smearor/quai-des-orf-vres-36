use crate::MoveRecord;
use crate::PublicEntry;
use crate::RevealSchedule;
use qdo36_material::Role;
use serde::Deserialize;
use serde::Serialize;

/// The ordered record of all committed moves.
///
/// The log is the single source of truth for the HUD column, the
/// `move_log` MCP tool and the replay mode. Fugitive's destinations are
/// stored truthfully - secrecy is enforced by *projections*, never by
/// hiding data inconsistently.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameLog {
    /// Committed moves in order.
    pub entries: Vec<MoveRecord>,
}

impl GameLog {
    /// The detectives' view: detective destinations are public, Fugitive's
    /// only on reveal rounds.
    ///
    /// Used by the HUD, the public MCP state and the move-log tool - the
    /// same projection everywhere, so secrecy cannot drift.
    pub fn fugitive_public_view(&self, reveal: &RevealSchedule) -> Vec<PublicEntry> {
        self.entries
            .iter()
            .map(|entry| PublicEntry {
                round: entry.round,
                ticket: entry.ticket,
                destination: match entry.actor {
                    Role::Detective(_) => Some(entry.kind.destination()),
                    Role::Fugitive => reveal.is_reveal(entry.round).then(|| entry.kind.destination()),
                },
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hop;
    use crate::MoveKind;
    use qdo36_map::StationId;
    use qdo36_map::Transport;
    use qdo36_material::PawnId;
    use qdo36_material::RoundNumber;
    use qdo36_material::TicketKind;

    fn record(actor: Role, round: u8, to: u16) -> MoveRecord {
        let hop = Hop {
            from: StationId::new(1),
            to: StationId::new(to),
            transport: Transport::Taxi,
        };
        MoveRecord {
            round: RoundNumber::new(round),
            actor,
            ticket: TicketKind::Taxi,
            kind: MoveKind::Single(hop),
        }
    }

    #[test]
    fn public_view_redacts_fugitive_until_reveal() {
        let log = GameLog {
            entries: vec![
                record(Role::Fugitive, 1, 10),
                record(Role::Fugitive, 3, 20),
                record(Role::Detective(PawnId::new(0)), 3, 30),
            ],
        };
        let view = log.fugitive_public_view(&RevealSchedule::classic());
        assert_eq!(view[0].destination, None, "round 1 stays hidden");
        assert_eq!(view[1].destination, Some(StationId::new(20)), "round 3 is revealed");
        assert_eq!(view[2].destination, Some(StationId::new(30)), "detectives are always public");
    }
}
