//! Bridge decks: the landing geometry at `bridge` stations -
//! which side of the deck a route enters from and how it docks.

pub mod deck;
pub mod near_point;
pub mod route;
pub mod side;

pub use deck::BridgeDeck;
pub use route::RoadRoute;
pub use side::DeckSide;
