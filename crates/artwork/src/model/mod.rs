//! Configuration model of the variants table (`variants.toml`).

mod asset;
mod inner_fill;
mod spec;
mod substitution;
mod text_fill;
mod variant;

pub use asset::Asset;
pub use inner_fill::InnerFill;
pub use spec::Spec;
pub use substitution::Substitution;
pub use text_fill::TextFill;
pub use variant::Variant;
