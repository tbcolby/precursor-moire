//! UI rendering for Moiré Generator.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use gam::{Gam, Point, Line, TextView, TextBounds, GlyphStyle};

use crate::app::{MoireApp, AppState, MenuField, PRESETS, RENDER_W, RENDER_H};

const SCREEN_W: i16 = 336;
const HEADER_H: i16 = 30;
const FOOTER_Y: i16 = 490;

fn draw_header(gam: &Gam, title: &str) {
    gam.draw_rectangle(Point::new(0, 0), Point::new(SCREEN_W, HEADER_H)).ok();
    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, 2), Point::new(SCREEN_W - 8, HEADER_H)),
    );
    tv.style = GlyphStyle::Bold;
    tv.invert = true;
    tv.text = String::from(title);
    gam.post_textview(&mut tv).ok();
}

fn draw_footer(gam: &Gam, text: &str) {
    gam.draw_line(Line::new(Point::new(0, FOOTER_Y), Point::new(SCREEN_W, FOOTER_Y))).ok();
    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, FOOTER_Y + 4), Point::new(SCREEN_W - 8, FOOTER_Y + 40)),
    );
    tv.style = GlyphStyle::Small;
    tv.text = String::from(text);
    gam.post_textview(&mut tv).ok();
}

pub fn draw(gam: &Gam, app: &MoireApp) {
    gam.clear().ok();

    match app.state {
        AppState::Menu => draw_menu(gam, app),
        AppState::Preview => draw_preview(gam, app),
        AppState::Gallery => draw_gallery(gam, app),
    }

    gam.redraw().ok();
}

fn draw_menu(gam: &Gam, app: &MoireApp) {
    draw_header(gam, "Moiré Generator");

    // Section: Layer A
    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, 38), Point::new(SCREEN_W - 8, 56)),
    );
    tv.style = GlyphStyle::Small;
    tv.text = String::from("— Layer A —");
    gam.post_textview(&mut tv).ok();

    let fields = [
        (MenuField::LayerAType, format!("Type:    < {} >", app.layer_a.kind.label()), 58),
        (MenuField::LayerASpacing, format!("Spacing: < {} >", app.layer_a.spacing), 88),
        (MenuField::LayerAAngle, format!("Angle:   < {}° >", app.layer_a.angle), 118),
    ];

    for (field, label, y) in &fields {
        draw_field(gam, app, *field, label, *y);
    }

    // Section: Layer B
    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, 156), Point::new(SCREEN_W - 8, 174)),
    );
    tv.style = GlyphStyle::Small;
    tv.text = String::from("— Layer B —");
    gam.post_textview(&mut tv).ok();

    let fields_b = [
        (MenuField::LayerBType, format!("Type:    < {} >", app.layer_b.kind.label()), 176),
        (MenuField::LayerBSpacing, format!("Spacing: < {} >", app.layer_b.spacing), 206),
        (MenuField::LayerBAngle, format!("Angle:   < {}° >", app.layer_b.angle), 236),
    ];

    for (field, label, y) in &fields_b {
        draw_field(gam, app, *field, label, *y);
    }

    // Buttons
    draw_field(gam, app, MenuField::Render, &String::from("[ Render Pattern ]"), 280);
    draw_field(gam, app, MenuField::Gallery, &String::from("[ Gallery ]"), 316);

    draw_footer(gam, "Arrows: navigate/adjust  Enter: select  Menu: quit");
}

fn draw_field(gam: &Gam, app: &MoireApp, field: MenuField, label: &str, y: i16) {
    let selected = app.menu_field == field;
    let h: i16 = 26;

    if selected {
        gam.draw_rectangle(
            Point::new(4, y - 2),
            Point::new(SCREEN_W - 4, y + h),
        ).ok();
    }

    let mut tv = TextView::new(
        TextBounds::BoundingBox(
            Point::new(12, y),
            Point::new(SCREEN_W - 12, y + h + 4),
        ),
    );
    tv.style = GlyphStyle::Regular;
    if selected {
        tv.invert = true;
    }
    tv.text = String::from(label);
    gam.post_textview(&mut tv).ok();
}

fn draw_preview(gam: &Gam, app: &MoireApp) {
    draw_header(gam, "Moiré Preview");

    // Render bitmap pixel-by-pixel using draw_rectangle for set pixels
    // For efficiency, draw horizontal runs of set pixels
    let content_y: i16 = HEADER_H + 2;
    let row_bytes = (RENDER_W / 8) as usize;

    for py in 0..RENDER_H as usize {
        let screen_y = content_y + py as i16;
        if screen_y >= FOOTER_Y { break; }

        let row_start = py * row_bytes;
        let mut x = 0i16;
        while (x as usize) < RENDER_W as usize {
            let byte_idx = row_start + (x as usize / 8);
            if byte_idx >= app.bitmap.len() { break; }
            let byte = app.bitmap[byte_idx];
            let bit = 7 - (x as usize % 8);

            if (byte >> bit) & 1 == 1 {
                // Find run length
                let run_start = x;
                while (x as i32) < RENDER_W {
                    let bi = row_start + (x as usize / 8);
                    if bi >= app.bitmap.len() { break; }
                    let b = app.bitmap[bi];
                    let bt = 7 - (x as usize % 8);
                    if (b >> bt) & 1 == 0 { break; }
                    x += 1;
                }
                // Draw the run as a line
                gam.draw_line(Line::new(
                    Point::new(run_start, screen_y),
                    Point::new(x - 1, screen_y),
                )).ok();
            } else {
                x += 1;
            }
        }
    }

    draw_footer(gam, "Enter/Left/Menu: back to menu");
}

fn draw_gallery(gam: &Gam, app: &MoireApp) {
    draw_header(gam, "Gallery — Presets");

    let start_y: i16 = 40;
    let line_h: i16 = 36;
    let visible = 11;

    let scroll = if app.gallery_cursor >= visible {
        app.gallery_cursor - visible + 1
    } else {
        0
    };

    for i in 0..PRESETS.len().min(visible) {
        let idx = scroll + i;
        if idx >= PRESETS.len() { break; }

        let preset = &PRESETS[idx];
        let y = start_y + (i as i16) * line_h;
        let selected = idx == app.gallery_cursor;

        if selected {
            gam.draw_rectangle(
                Point::new(4, y - 2),
                Point::new(SCREEN_W - 4, y + line_h - 6),
            ).ok();
        }

        let label = format!("{} ({}/{})",
            preset.name, preset.a.kind.label(), preset.b.kind.label());

        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(12, y),
                Point::new(SCREEN_W - 12, y + line_h),
            ),
        );
        tv.style = GlyphStyle::Regular;
        if selected {
            tv.invert = true;
        }
        tv.text = label;
        gam.post_textview(&mut tv).ok();
    }

    draw_footer(gam, "Up/Down: browse  Enter: render  Left: back");
}
