//! `noise` - the shared deterministic hash the block passes
//! roll their variation from.

/// Deterministic hash noise in `0.0..1.0` (splitmix64 over
/// seed/ring/edge/slot/channel). `ring`/`edge` are the block's
/// placement coordinates, `channel` picks the roll's dimension.
pub(crate) fn noise(seed: u64, ring: usize, edge: usize, slot: usize, channel: u64) -> f64 {
    let mut z = seed
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(channel << 56)
        .wrapping_add((ring as u64) << 40)
        .wrapping_add((edge as u64) << 20)
        .wrapping_add(slot as u64);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}
