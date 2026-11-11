//! `style` - the style drawer's miniature-artwork picker.

use crate::mcp_bridge;
use crate::ui::style::MapStyle;
use gtk4::Align;
use gtk4::Orientation;
use gtk4::prelude::*;
use std::f64::consts::TAU;

/// The style drawer content: one button per artwork style showing
/// a live miniature of its palette plus the style's name.
pub(crate) fn style_list() -> gtk4::Widget {
    let list = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();
    let styles = [("Comic", MapStyle::Comic), ("Belle Époque", MapStyle::BelleEpoque), ("Neon", MapStyle::Neon)];
    let current = mcp_bridge::current_style().unwrap_or_default();
    for (label, style) in styles {
        let row = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(4).build();
        row.append(&style_preview(style));
        row.append(&gtk4::Label::new(Some(label)));
        let button = gtk4::Button::builder().child(&row).build();
        if style == current {
            button.add_css_class("checked");
        }
        button.connect_clicked(move |_| mcp_bridge::set_map_style(style));
        list.append(&button);
    }
    list.upcast()
}

/// A miniature scene painted in the style's palette - paper,
/// houses, a water band, an accent route and a station dot.
fn style_preview(style: MapStyle) -> gtk4::DrawingArea {
    let palette = style.palette();
    let area = gtk4::DrawingArea::builder()
        .content_width(64)
        .content_height(40)
        .hexpand(false)
        .halign(Align::Center)
        .build();
    area.set_draw_func(move |_, ctx, w, h| {
        let (w, h) = (f64::from(w), f64::from(h));
        palette.paper().apply(ctx);
        ctx.rectangle(0.0, 0.0, w, h);
        ctx.fill().ok();
        // Houses on the paper field.
        palette.building().apply(ctx);
        for (x, y, hw, hh) in [(0.10, 0.14, 0.22, 0.26), (0.40, 0.08, 0.26, 0.34), (0.72, 0.18, 0.20, 0.24)] {
            ctx.rectangle(w * x, h * y, w * hw, h * hh);
        }
        ctx.fill().ok();
        // The river band crossing the lower third.
        palette.water_fill().apply(ctx);
        ctx.move_to(0.0, h * 0.72);
        ctx.line_to(w, h * 0.58);
        ctx.line_to(w, h * 0.86);
        ctx.line_to(0.0, h * 0.98);
        ctx.close_path();
        ctx.fill().ok();
        // An accent route plus a station disc.
        ctx.set_line_width(2.2);
        palette.accent().apply(ctx);
        ctx.move_to(w * 0.08, h * 0.62);
        ctx.line_to(w * 0.92, h * 0.30);
        ctx.stroke().ok();
        palette.station().apply(ctx);
        ctx.arc(w * 0.5, h * 0.46, 4.0, 0.0, TAU);
        ctx.fill().ok();
        ctx.set_line_width(1.4);
        palette.ink().apply(ctx);
        ctx.stroke().ok();
    });
    area
}
