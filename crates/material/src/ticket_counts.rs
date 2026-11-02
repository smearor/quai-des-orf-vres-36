use crate::TicketKind;
use serde::Deserialize;
use serde::Serialize;

/// One player's ticket inventory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketCounts {
    /// Remaining taxi tickets.
    pub taxi: u8,
    /// Remaining bus tickets.
    pub bus: u8,
    /// Remaining metro tickets.
    pub metro: u8,
    /// Remaining black tickets (Fugitive only).
    pub black: u8,
    /// Remaining double tickets (Fugitive only).
    pub double: u8,
}

impl TicketCounts {
    /// The remaining count of a ticket kind.
    pub fn get(&self, kind: TicketKind) -> u8 {
        match kind {
            TicketKind::Taxi => self.taxi,
            TicketKind::Bus => self.bus,
            TicketKind::Metro => self.metro,
            TicketKind::Black => self.black,
            TicketKind::Double => self.double,
        }
    }

    /// Adds one ticket of `kind` - spent detective tickets feed
    /// Fugitive's pool (saturates at [`Self::UNLIMITED`]).
    pub fn gain(&mut self, kind: TicketKind) {
        let slot = match kind {
            TicketKind::Taxi => &mut self.taxi,
            TicketKind::Bus => &mut self.bus,
            TicketKind::Metro => &mut self.metro,
            TicketKind::Black => &mut self.black,
            TicketKind::Double => &mut self.double,
        };
        *slot = slot.saturating_add(1);
    }

    /// Decrements a ticket kind by one; returns `false` when none left.
    pub fn spend(&mut self, kind: TicketKind) -> bool {
        let slot = match kind {
            TicketKind::Taxi => &mut self.taxi,
            TicketKind::Bus => &mut self.bus,
            TicketKind::Metro => &mut self.metro,
            TicketKind::Black => &mut self.black,
            TicketKind::Double => &mut self.double,
        };
        if *slot == 0 {
            false
        } else {
            *slot -= 1;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spend_decrements_and_blocks_at_zero() {
        let mut counts = TicketCounts { taxi: 1, ..Default::default() };
        assert!(counts.spend(TicketKind::Taxi));
        assert_eq!(counts.get(TicketKind::Taxi), 0);
        assert!(!counts.spend(TicketKind::Taxi));
    }
}
