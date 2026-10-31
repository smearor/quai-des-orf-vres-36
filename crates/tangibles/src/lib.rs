//! Tangible input for Quai des Orfèvres 36.
//!
//! Talks to the `tangible-objects` daemon over its GraphQL interface
//! (pattern registration, `tangibleEvents` subscription, `deviceInfo`
//! and `touchPoints` queries) and derives the Eiffel tower anchor pose
//! plus the map-space transform from it.
