//! Water geometry: river bands, banks and course smoothing in map mm.

/// Half the river band width - shared by the water bands and the
/// bridge-station decks, so every part of the scene agrees how
/// wide the river is.
pub const WATER_HALF_WIDTH: Millimeters = Millimeters::new(3.5);

use cairo::Context;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

use crate::ui::style::PxPos;

/// Corner-cutting subdivision (Chaikin): each round replaces every
/// segment by points at 1/4 and 3/4, rounding corners and roughly
/// doubling the point count - denser spline checkpoints and no
/// miter spikes at sharp river bends. `pinned` lists point indices
/// (of the original polyline) that stay on the course exactly -
/// bridge stations sit on those vertices, so the water keeps
/// flowing right under their markers.
pub fn smooth_polyline(points: &[Vec2<Millimeters>], rounds: usize, pinned: &[usize]) -> Vec<Vec2<Millimeters>> {
    let mut flags: Vec<bool> = points.iter().enumerate().map(|(index, _)| pinned.contains(&index)).collect();
    let mut current = points.to_vec();
    for _ in 0..rounds {
        if current.len() < 3 {
            break;
        }
        let mut next = Vec::with_capacity(current.len() * 2);
        let mut next_flags = Vec::with_capacity(flags.len() * 2);
        next.push(current[0]);
        next_flags.push(flags[0]);
        for (index, window) in current.windows(2).enumerate() {
            let (from, to) = (window[0], window[1]);
            next.push(Vec2::new(from.x * 0.75 + to.x * 0.25, from.y * 0.75 + to.y * 0.25));
            next.push(Vec2::new(from.x * 0.25 + to.x * 0.75, from.y * 0.25 + to.y * 0.75));
            next_flags.extend_from_slice(&[false, false]);
            if flags.get(index + 1).copied().unwrap_or(false) {
                next.push(to);
                next_flags.push(true);
            }
        }
        if let (Some(last), Some(flag)) = (current.last(), flags.last()) {
            next.push(*last);
            next_flags.push(*flag);
        }
        (current, flags) = (next, next_flags);
    }
    current
}

/// The closed band between two bank lines - the filled river body.
pub fn band_path(ctx: &Context, left_bank: &[PxPos], right_bank: &[PxPos]) {
    ctx.new_path();
    let Some(first) = left_bank.first() else { return };
    ctx.move_to(first.x, first.y);
    for point in &left_bank[1..] {
        ctx.line_to(point.x, point.y);
    }
    for point in right_bank.iter().rev() {
        ctx.line_to(point.x, point.y);
    }
    ctx.close_path();
}
