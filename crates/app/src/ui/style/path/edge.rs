//! Edge paths: lane-offset polylines, waves and splines in widget px.

use cairo::Context;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;

use crate::ui::style::EdgeSpec;
use crate::ui::style::PxPos;

/// Cairo edge paths: `ctx.edge_path(..)` appends the lane-offset
/// polyline, `ctx.px_polyline_path(..)` traces a px polyline.
pub trait EdgePathExt {
    /// Builds the edge path: the `route` polyline (or the straight
    /// `from` -> `to` chord) shifted per segment by `offsets` -
    /// parallel metro lines draw side by side in their own colors.
    fn edge_path(&self, edge: &EdgeSpec<'_>);
    /// Strokes a polyline already transformed to widget px.
    fn px_polyline_path(&self, points: &[PxPos]);
}

impl EdgePathExt for Context {
    fn edge_path(&self, edge: &EdgeSpec<'_>) {
        if edge.wavy && edge.route.len() >= 2 {
            wavy_path(
                self,
                &edge.route,
                edge.px_per_mm.scale(Millimeters::new(32.0)),
                edge.px_per_mm.scale(Millimeters::new(1.2)),
                edge.px_per_mm.scale(edge.wave_phase),
            );
            return;
        }
        if edge.smooth && edge.route.len() >= 3 {
            smooth_path(self, &edge.route);
            return;
        }
        if edge.route.is_empty() {
            let offset = edge.offsets.first().map(|offset| offset.as_f64()).unwrap_or(0.0);
            let (dx, dy) = (edge.to.x - edge.from.x, edge.to.y - edge.from.y);
            let len = (dx * dx + dy * dy).sqrt().max(1e-6);
            self.move_to(edge.from.x - dy / len * offset, edge.from.y + dx / len * offset);
            self.line_to(edge.to.x - dy / len * offset, edge.to.y + dx / len * offset);
            return;
        }
        if edge.route.len() < 2 {
            return;
        }
        // Per-vertex lane position: the wider corridor keeps its lane
        // right up to the vertex (the bigger |offset| wins), so a
        // lane disappearing into a thinner corridor peels off on its
        // own hop instead of gliding across neighbor lanes. Equal
        // offsets join in a miter (via vertices on one corridor).
        let seg_shift = |segment: usize| -> (f64, f64) {
            let offset = edge.offsets.get(segment).map(|offset| offset.as_f64()).unwrap_or(0.0);
            let (a, b) = (edge.route[segment], edge.route[segment + 1]);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let len = dx.hypot(dy).max(1e-6);
            (-dy / len * offset, dx / len * offset)
        };
        let last = edge.route.len() - 1;
        let vertex_shift = |index: usize| -> (f64, f64) {
            if index == 0 {
                seg_shift(0)
            } else if index == last {
                seg_shift(last - 1)
            } else {
                let (ax, ay) = seg_shift(index - 1);
                let (bx, by) = seg_shift(index);
                let (wa, wb) = (ax.hypot(ay), bx.hypot(by));
                if (wa - wb).abs() < 1e-9 {
                    ((ax + bx) / 2.0, (ay + by) / 2.0)
                } else if wa > wb {
                    (ax, ay)
                } else {
                    (bx, by)
                }
            }
        };
        let (nx, ny) = vertex_shift(0);
        self.move_to(edge.route[0].x + nx, edge.route[0].y + ny);
        for index in 1..edge.route.len() {
            let (nx, ny) = vertex_shift(index);
            self.line_to(edge.route[index].x + nx, edge.route[index].y + ny);
        }
    }

    fn px_polyline_path(&self, points: &[PxPos]) {
        self.new_path();
        for (index, point) in points.iter().enumerate() {
            if index == 0 {
                self.move_to(point.x, point.y);
            } else {
                self.line_to(point.x, point.y);
            }
        }
    }
}

/// A sine wave traced along `points` - the Batobus boat wake: the
/// polyline is sampled at fine arc-length steps and each sample
/// shifts perpendicular to the local segment direction. `phase`
/// offsets the wave's arc position, so edges sharing a stretch
/// trace the same wave.
fn wavy_path(ctx: &Context, points: &[PxPos], wavelength: Pixels, amplitude: Pixels, phase: Pixels) {
    let (wavelength, amplitude, phase) = (wavelength.as_f64(), amplitude.as_f64(), phase.as_f64());
    ctx.new_path();
    let Some(first) = points.first() else { return };
    ctx.move_to(first.x, first.y);
    let step = (wavelength / 16.0).max(0.15);
    let mut sample_arc = step;
    let mut seg_start_arc = 0.0;
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let seg_len = dx.hypot(dy);
        if seg_len < 1e-6 {
            continue;
        }
        let (ux, uy) = (dx / seg_len, dy / seg_len);
        while sample_arc < seg_start_arc + seg_len {
            let t = sample_arc - seg_start_arc;
            let wave = amplitude * ((sample_arc + phase) / wavelength * std::f64::consts::TAU).sin();
            ctx.line_to(a.x + ux * t - uy * wave, a.y + uy * t + ux * wave);
            sample_arc += step;
        }
        seg_start_arc += seg_len;
    }
    if let Some(last) = points.last() {
        ctx.line_to(last.x, last.y);
    }
}

/// A smooth spline through `points` (Catmull-Rom -> cubic Bézier) -
/// the Batobus boat course curves along the river between piers.
/// The first and last hop draw straight: they are the short pier
/// stubs off the waterline, and the off-water station may not pull
/// the spline tangents or the curve loops at every pier.
fn smooth_path(ctx: &Context, points: &[PxPos]) {
    ctx.new_path();
    let Some(first) = points.first() else { return };
    ctx.move_to(first.x, first.y);
    if points.len() < 4 {
        for point in &points[1..] {
            ctx.line_to(point.x, point.y);
        }
        return;
    }
    ctx.line_to(points[1].x, points[1].y);
    let last = points.len() - 2;
    let at = |index: isize| points[index.clamp(1, last as isize) as usize];
    for i in 1..last {
        let (p0, p1, p2, p3) = (at(i as isize - 1), at(i as isize), at(i as isize + 1), at(i as isize + 2));
        ctx.curve_to(
            p1.x + (p2.x - p0.x) / 6.0,
            p1.y + (p2.y - p0.y) / 6.0,
            p2.x - (p3.x - p1.x) / 6.0,
            p2.y - (p3.y - p1.y) / 6.0,
            p2.x,
            p2.y,
        );
    }
    ctx.line_to(points[last + 1].x, points[last + 1].y);
}
