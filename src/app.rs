//! State machine for Moiré Generator.
//!
//! States:
//!   Menu      — choose pattern types and parameters for two layers
//!   Preview   — render the moiré pattern
//!   Gallery   — preset patterns

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::patterns::{self, Layer, PatternKind};

const KEY_UP: char = '\u{F700}';
const KEY_DOWN: char = '\u{F701}';
const KEY_LEFT: char = '\u{F702}';
const KEY_RIGHT: char = '\u{F703}';
const KEY_ENTER: char = '\u{000D}';
const KEY_MENU: char = '\u{2234}';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Menu,
    Preview,
    Gallery,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuField {
    LayerAType,
    LayerASpacing,
    LayerAAngle,
    LayerBType,
    LayerBSpacing,
    LayerBAngle,
    Render,
    Gallery,
}

/// A preset moiré combination.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub a: Layer,
    pub b: Layer,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Classic Lines",
        a: Layer { kind: PatternKind::Lines, spacing: 8, angle: 0, offset_x: 0, offset_y: 0 },
        b: Layer { kind: PatternKind::Lines, spacing: 8, angle: 5, offset_x: 0, offset_y: 0 },
    },
    Preset {
        name: "Concentric Ripple",
        a: Layer { kind: PatternKind::Circles, spacing: 6, angle: 0, offset_x: -20, offset_y: 0 },
        b: Layer { kind: PatternKind::Circles, spacing: 6, angle: 0, offset_x: 20, offset_y: 0 },
    },
    Preset {
        name: "Fabric Weave",
        a: Layer { kind: PatternKind::Grid, spacing: 10, angle: 0, offset_x: 0, offset_y: 0 },
        b: Layer { kind: PatternKind::Grid, spacing: 10, angle: 15, offset_x: 0, offset_y: 0 },
    },
    Preset {
        name: "Radial Burst",
        a: Layer { kind: PatternKind::Radial, spacing: 8, angle: 0, offset_x: -30, offset_y: 0 },
        b: Layer { kind: PatternKind::Radial, spacing: 8, angle: 0, offset_x: 30, offset_y: 0 },
    },
    Preset {
        name: "Zebra",
        a: Layer { kind: PatternKind::Lines, spacing: 6, angle: 0, offset_x: 0, offset_y: 0 },
        b: Layer { kind: PatternKind::Lines, spacing: 7, angle: 0, offset_x: 0, offset_y: 0 },
    },
    Preset {
        name: "Hypnotic",
        a: Layer { kind: PatternKind::Circles, spacing: 5, angle: 0, offset_x: 0, offset_y: -40 },
        b: Layer { kind: PatternKind::Circles, spacing: 5, angle: 0, offset_x: 0, offset_y: 40 },
    },
    Preset {
        name: "Grid Rotation",
        a: Layer { kind: PatternKind::Grid, spacing: 12, angle: 0, offset_x: 0, offset_y: 0 },
        b: Layer { kind: PatternKind::Grid, spacing: 12, angle: 7, offset_x: 0, offset_y: 0 },
    },
    Preset {
        name: "Mixed Media",
        a: Layer { kind: PatternKind::Lines, spacing: 8, angle: 45, offset_x: 0, offset_y: 0 },
        b: Layer { kind: PatternKind::Circles, spacing: 10, angle: 0, offset_x: 0, offset_y: 0 },
    },
];

// Render dimensions — content area within header/footer
pub const RENDER_W: i32 = 336;
pub const RENDER_H: i32 = 456; // 490-34 (header to footer)

pub struct MoireApp {
    pub state: AppState,
    pub needs_redraw: bool,

    pub menu_field: MenuField,
    pub layer_a: Layer,
    pub layer_b: Layer,

    // Rendered bitmap (XOR of both layers)
    pub bitmap: Vec<u8>,

    pub gallery_cursor: usize,
}

impl MoireApp {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
            needs_redraw: true,
            menu_field: MenuField::LayerAType,
            layer_a: Layer::default_a(),
            layer_b: Layer::default_b(),
            bitmap: Vec::new(),
            gallery_cursor: 0,
        }
    }

    fn render_moire(&mut self) {
        let a = patterns::render_layer(&self.layer_a, RENDER_W, RENDER_H);
        let b = patterns::render_layer(&self.layer_b, RENDER_W, RENDER_H);
        self.bitmap = patterns::xor_bitmaps(&a, &b);
    }

    pub fn handle_key(&mut self, key: char) -> bool {
        self.needs_redraw = true;
        match self.state {
            AppState::Menu => self.handle_menu(key),
            AppState::Preview => self.handle_preview(key),
            AppState::Gallery => self.handle_gallery(key),
        }
    }

    fn handle_menu(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => return false,
            KEY_UP => {
                self.menu_field = match self.menu_field {
                    MenuField::LayerAType => MenuField::Gallery,
                    MenuField::LayerASpacing => MenuField::LayerAType,
                    MenuField::LayerAAngle => MenuField::LayerASpacing,
                    MenuField::LayerBType => MenuField::LayerAAngle,
                    MenuField::LayerBSpacing => MenuField::LayerBType,
                    MenuField::LayerBAngle => MenuField::LayerBSpacing,
                    MenuField::Render => MenuField::LayerBAngle,
                    MenuField::Gallery => MenuField::Render,
                };
            }
            KEY_DOWN => {
                self.menu_field = match self.menu_field {
                    MenuField::LayerAType => MenuField::LayerASpacing,
                    MenuField::LayerASpacing => MenuField::LayerAAngle,
                    MenuField::LayerAAngle => MenuField::LayerBType,
                    MenuField::LayerBType => MenuField::LayerBSpacing,
                    MenuField::LayerBSpacing => MenuField::LayerBAngle,
                    MenuField::LayerBAngle => MenuField::Render,
                    MenuField::Render => MenuField::Gallery,
                    MenuField::Gallery => MenuField::LayerAType,
                };
            }
            KEY_LEFT | KEY_RIGHT => {
                let delta = if key == KEY_RIGHT { 1 } else { -1 };
                match self.menu_field {
                    MenuField::LayerAType => {
                        self.layer_a.kind = if delta > 0 { self.layer_a.kind.next() } else { self.layer_a.kind.prev() };
                    }
                    MenuField::LayerASpacing => {
                        self.layer_a.spacing = (self.layer_a.spacing + delta).max(3).min(30);
                    }
                    MenuField::LayerAAngle => {
                        self.layer_a.angle = ((self.layer_a.angle + delta * 5) % 360 + 360) % 360;
                    }
                    MenuField::LayerBType => {
                        self.layer_b.kind = if delta > 0 { self.layer_b.kind.next() } else { self.layer_b.kind.prev() };
                    }
                    MenuField::LayerBSpacing => {
                        self.layer_b.spacing = (self.layer_b.spacing + delta).max(3).min(30);
                    }
                    MenuField::LayerBAngle => {
                        self.layer_b.angle = ((self.layer_b.angle + delta * 5) % 360 + 360) % 360;
                    }
                    _ => {}
                }
            }
            KEY_ENTER => match self.menu_field {
                MenuField::Render => {
                    self.render_moire();
                    self.state = AppState::Preview;
                }
                MenuField::Gallery => {
                    self.state = AppState::Gallery;
                }
                _ => {}
            },
            _ => {}
        }
        true
    }

    fn handle_preview(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT | KEY_ENTER => {
                self.state = AppState::Menu;
            }
            _ => {}
        }
        true
    }

    fn handle_gallery(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT => {
                self.state = AppState::Menu;
            }
            KEY_UP => {
                if self.gallery_cursor > 0 {
                    self.gallery_cursor -= 1;
                }
            }
            KEY_DOWN => {
                if self.gallery_cursor < PRESETS.len() - 1 {
                    self.gallery_cursor += 1;
                }
            }
            KEY_ENTER => {
                let preset = PRESETS[self.gallery_cursor];
                self.layer_a = preset.a;
                self.layer_b = preset.b;
                self.render_moire();
                self.state = AppState::Preview;
            }
            _ => {}
        }
        true
    }
}
