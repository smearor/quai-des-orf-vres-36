//! Scripted games: deal, validation error paths, apply side effects,
//! win conditions, undo/redo and the secrecy projection.

use qdo36_map::GameMap;
use qdo36_map::Transport;
use qdo36_material::PawnId;
use qdo36_material::Role;
use qdo36_material::RoundNumber;
use qdo36_material::TicketCounts;
use qdo36_material::TicketKind;
use qdo36_moves::GameOutcome;
use qdo36_moves::GamePhase;
use qdo36_moves::Hop;
use qdo36_moves::MoveError;
use qdo36_moves::MoveKind;
use qdo36_moves::MoveOutcome;
use qdo36_moves::MoveRequest;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::GameConfig;
use crate::GameSession;

/// A dealt session on the bundled Paris map with a fixed seed
/// (reproducible positions).
fn dealt() -> (GameSession, GameMap) {
    let map = GameMap::bundled().unwrap_or_else(|_| panic!("bundled map must load"));
    let config = GameConfig::for_map(map.meta());
    let mut rng = StdRng::seed_from_u64(36);
    let session = GameSession::deal(config, &map, &mut rng).unwrap_or_else(|err| panic!("deal must succeed: {err}"));
    (session, map)
}

/// A session past the Deal phase, Fugitive to move.
fn running() -> (GameSession, GameMap) {
    let (mut session, map) = dealt();
    session.begin().unwrap_or_else(|_| panic!("begin must succeed"));
    (session, map)
}

/// A single-hop request for the active role along a given hop.
fn request(session: &GameSession, hop: Hop) -> MoveRequest {
    let ticket = match hop.transport {
        Transport::Taxi => TicketKind::Taxi,
        Transport::Bus => TicketKind::Bus,
        Transport::Metro => TicketKind::Metro,
        Transport::Batobus | Transport::Underground => TicketKind::Black,
    };
    MoveRequest {
        actor: session.state.active,
        ticket,
        kind: MoveKind::Single(hop),
    }
}

#[test]
fn deal_assigns_seats_tickets_and_hidden_fugitive() {
    let (session, map) = dealt();
    let seats = session.config.detectives.count() as usize;
    assert_eq!(session.state.positions.len(), seats, "every detective is seated");
    let mut distinct = session.state.positions.values().copied().collect::<Vec<_>>();
    distinct.push(session.state.fugitive_pos);
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), seats + 1, "every dealt station is distinct");
    for station in &distinct {
        assert!(map.start_stations().any(|s| s.id == *station), "{station} is a start station");
    }
    assert_eq!(session.state.phase, GamePhase::Deal);
    assert_eq!(session.state.tickets[&Role::Detective(PawnId::new(0))], session.config.detective_stock);
    assert_eq!(session.state.tickets[&Role::Fugitive].black, session.config.fugitive_stock.black);
}

#[test]
fn deal_honors_configured_stocks() {
    let map = GameMap::bundled().unwrap_or_else(|_| panic!());
    let mut config = GameConfig::for_map(map.meta());
    config.detective_stock.taxi = 3;
    config.fugitive_stock.black = 1;
    let session = GameSession::deal(config, &map, &mut StdRng::seed_from_u64(36)).unwrap_or_else(|_| panic!());
    assert_eq!(session.state.tickets[&Role::Detective(PawnId::new(0))].taxi, 3);
    assert_eq!(session.state.tickets[&Role::Fugitive].black, 1);
}

#[test]
fn deterministic_seed_repeats_the_deal() {
    let map = GameMap::bundled().unwrap_or_else(|_| panic!());
    let config = GameConfig::for_map(map.meta());
    let a = GameSession::deal(config.clone(), &map, &mut StdRng::seed_from_u64(7)).unwrap_or_else(|_| panic!());
    let b = GameSession::deal(config, &map, &mut StdRng::seed_from_u64(7)).unwrap_or_else(|_| panic!());
    assert_eq!(a.state.positions, b.state.positions);
    assert_eq!(a.state.fugitive_pos, b.state.fugitive_pos);
}

#[test]
fn moves_are_rejected_during_deal() {
    let (mut session, map) = dealt();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    let err = session.request_move(&map, &request(&session, hop)).err().unwrap_or_else(|| panic!("must fail"));
    assert!(matches!(err, MoveError::WrongPhase { .. }));
}

#[test]
fn wrong_actor_is_rejected() {
    let (mut session, map) = running();
    let other = Role::Detective(PawnId::new(0));
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    let request = MoveRequest {
        actor: other,
        ticket: TicketKind::Taxi,
        kind: MoveKind::Single(hop),
    };
    let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
    assert_eq!(
        err,
        MoveError::NotPlayersTurn {
            expected: Role::Fugitive,
            actual: other
        }
    );
}

#[test]
fn detectives_cannot_use_fugitive_only_tickets() {
    let (mut session, map) = running();
    // Fugitive moves first so a detective becomes active.
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let pawn = match session.state.active {
        Role::Detective(pawn) => pawn,
        Role::Fugitive => panic!("a detective must be active"),
    };
    let hop = session.legal_destinations(&map, Role::Detective(pawn))[0];
    let request = MoveRequest {
        actor: Role::Detective(pawn),
        ticket: TicketKind::Black,
        kind: MoveKind::Single(hop),
    };
    let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
    assert_eq!(err, MoveError::FugitiveOnlyTicket(TicketKind::Black));
}

#[test]
fn double_ticket_requires_the_double_shape() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    let request = MoveRequest {
        actor: Role::Fugitive,
        ticket: TicketKind::Double,
        kind: MoveKind::Single(hop),
    };
    let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
    assert!(matches!(
        err,
        MoveError::TicketShapeMismatch {
            ticket: TicketKind::Double,
            ..
        }
    ));
}

#[test]
fn batobus_demands_the_black_ticket() {
    let (mut session, map) = running();
    let pos = session.state.fugitive_pos;
    let river = map
        .neighbors(pos)
        .iter()
        .find(|n| matches!(n.transport, Transport::Batobus | Transport::Underground));
    let Some(river) = river else { return };
    let request = MoveRequest {
        actor: Role::Fugitive,
        ticket: TicketKind::Taxi,
        kind: MoveKind::Single(Hop {
            from: pos,
            to: river.station,
            transport: river.transport,
        }),
    };
    let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
    assert_eq!(err, MoveError::BatobusRequiresBlack);
}

#[test]
fn applied_moves_update_state_log_and_turn() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    let outcome = session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let record = match outcome {
        MoveOutcome::Applied(record) => record,
        MoveOutcome::Finished { .. } => panic!("one move cannot finish the game"),
    };
    assert_eq!(session.state.fugitive_pos, hop.to);
    assert_eq!(session.state.log.entries, vec![record]);
    assert!(matches!(session.state.active, Role::Detective(_)), "Fugitive hands over to detectives");
    assert!(session.can_undo());
}

#[test]
fn occupied_destinations_are_rejected() {
    let (mut session, map) = running();
    // Find a detective station adjacent to Fugitive's position.
    let Some(hop) = map
        .neighbors(session.state.fugitive_pos)
        .iter()
        .find(|n| session.state.positions.values().any(|pos| *pos == n.station))
        .map(|n| Hop {
            from: session.state.fugitive_pos,
            to: n.station,
            transport: n.transport,
        })
    else {
        return;
    };
    let request = MoveRequest {
        actor: Role::Fugitive,
        ticket: TicketKind::Black,
        kind: MoveKind::Single(hop),
    };
    let err = session.request_move(&map, &request).err().unwrap_or_else(|| panic!("must fail"));
    assert!(matches!(err, MoveError::DestinationOccupied(..)));
}

#[test]
fn undo_restores_state_and_returns_a_plan() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let detective = session.state.active;
    let hop = session.legal_destinations(&map, detective)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let plan = session.undo().unwrap_or_else(|| panic!("undo must work"));
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(session.state.log.entries.len(), 1);
    assert!(session.can_redo());
    let plan = session.redo().unwrap_or_else(|| panic!("redo must work"));
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(session.state.log.entries.len(), 2);
}

#[test]
fn undoing_a_fugitive_move_yields_an_empty_plan() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let plan = session.undo().unwrap_or_else(|| panic!("undo must work"));
    assert!(plan.steps.is_empty(), "fugitive has no physical pawn");
}

#[test]
fn landing_on_fugitive_wins_for_the_detectives() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    // Teleport a detective next to Fugitive for the scripted catch.
    let fugitive = session.state.fugitive_pos;
    let pawn = match session.state.active {
        Role::Detective(pawn) => pawn,
        Role::Fugitive => panic!(),
    };
    let adjacent = map
        .neighbors(fugitive)
        .iter()
        .find(|n| session.station_free(n.station))
        .unwrap_or_else(|| panic!("adjacent station"));
    session.state.positions.insert(pawn, adjacent.station);
    let request = MoveRequest {
        actor: Role::Detective(pawn),
        ticket: match adjacent.transport {
            Transport::Taxi => TicketKind::Taxi,
            Transport::Bus => TicketKind::Bus,
            Transport::Metro => TicketKind::Metro,
            Transport::Batobus | Transport::Underground => TicketKind::Black,
        },
        kind: MoveKind::Single(Hop {
            from: adjacent.station,
            to: fugitive,
            transport: adjacent.transport,
        }),
    };
    let outcome = session.request_move(&map, &request).unwrap_or_else(|err| panic!("{err}"));
    match outcome {
        MoveOutcome::Finished { outcome, .. } => {
            assert!(matches!(outcome, GameOutcome::DetectivesWin { caught_at } if caught_at == fugitive));
        }
        MoveOutcome::Applied(_) => panic!("the capture must end the game"),
    }
    assert_eq!(session.state.phase, GamePhase::Finished);
}

#[test]
fn detective_tickets_feed_the_fugitive_pool() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let before = session.state.tickets[&Role::Fugitive].taxi;
    let hop = session
        .legal_destinations(&map, session.state.active)
        .iter()
        .find(|h| h.transport == Transport::Taxi)
        .copied();
    let Some(hop) = hop else { return };
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        session.state.tickets[&Role::Fugitive].taxi,
        before.saturating_add(1),
        "the spent taxi lands in Fugitive's pool"
    );
}

#[test]
fn public_view_hides_fugitive_until_reveal() {
    let (mut session, map) = running();
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let schedule = session.config.reveal_schedule();
    let view = session.state.log.fugitive_public_view(&schedule);
    assert_eq!(view[0].destination.is_none(), !schedule.is_reveal(session.state.round));
}

#[test]
fn reveal_round_surfaces_the_position() {
    let (mut session, map) = running();
    // Scripted: the next move is logged in reveal round 3.
    session.state.round = RoundNumber::new(3);
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    let view = session.state.log.fugitive_public_view(&session.config.reveal_schedule());
    assert_eq!(view[0].destination, Some(hop.to), "a reveal-round entry is public");
}

#[test]
fn double_move_spends_two_hops_in_one_turn() {
    let (mut session, map) = running();
    let first = session.legal_destinations(&map, Role::Fugitive)[0];
    let second = map.neighbors(first.to).iter().find(|n| session.station_free(n.station)).map(|n| Hop {
        from: first.to,
        to: n.station,
        transport: n.transport,
    });
    let Some(second) = second else { return };
    let request = MoveRequest {
        actor: Role::Fugitive,
        ticket: TicketKind::Double,
        kind: MoveKind::Double { first, second },
    };
    session.request_move(&map, &request).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(session.state.fugitive_pos, second.to, "the second hop decides the position");
    assert_eq!(session.state.log.entries.len(), 1, "a double move is one log entry");
    assert_eq!(session.state.tickets[&Role::Fugitive].double, session.config.fugitive_stock.double - 1);
}

#[test]
fn stuck_detectives_are_skipped_in_turn_order() {
    let (mut session, map) = running();
    session
        .request_move(&map, &request(&session, session.legal_destinations(&map, Role::Fugitive)[0]))
        .unwrap_or_else(|err| panic!("{err}"));
    // Immobilize the detective who follows the active one.
    let seats: Vec<PawnId> = session.state.positions.keys().copied().collect();
    let current = match session.state.active {
        Role::Detective(pawn) => pawn,
        Role::Fugitive => panic!("a detective must be active"),
    };
    let next_seat = seats.iter().find(|seat| **seat > current).copied();
    let Some(stuck_pawn) = next_seat else { return };
    session.state.tickets.insert(Role::Detective(stuck_pawn), TicketCounts::default());
    let hop = session.legal_destinations(&map, session.state.active)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    assert_ne!(session.state.active, Role::Detective(stuck_pawn), "a stuck detective is skipped");
}

#[test]
fn allow_undo_false_blocks_the_stacks() {
    let map = GameMap::bundled().unwrap_or_else(|_| panic!());
    let mut config = GameConfig::for_map(map.meta());
    config.allow_undo = false;
    let mut session = GameSession::deal(config, &map, &mut StdRng::seed_from_u64(36)).unwrap_or_else(|_| panic!("deal"));
    session.begin().unwrap_or_else(|_| panic!("begin"));
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    assert!(!session.can_undo(), "the config gate wins over a non-empty stack");
    assert!(session.undo().is_none());
}

#[test]
fn a_new_move_clears_the_redo_stack() {
    let (mut session, map) = running();
    session
        .request_move(&map, &request(&session, session.legal_destinations(&map, Role::Fugitive)[0]))
        .unwrap_or_else(|err| panic!("{err}"));
    session.undo().unwrap_or_else(|| panic!("undo"));
    assert!(session.can_redo());
    let hop = session.legal_destinations(&map, Role::Fugitive)[0];
    session.request_move(&map, &request(&session, hop)).unwrap_or_else(|err| panic!("{err}"));
    assert!(!session.can_redo(), "a fresh move discards the future");
}
