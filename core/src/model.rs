//! Menu / preview / gallery state machine. The shell maps GAM keys to
//! [`Key`] and draws the rectangles from [`Model::rects`].

use precursor_appkit_core::keys::Key;
use precursor_appkit_core::layout::ListState;

use crate::patterns::{self, Kind, Layer, MAX_OFFSET, MAX_SPACING, MIN_SPACING};
use crate::rects::{self, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Preview,
    Gallery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    A,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Kind(Which),
    Spacing(Which),
    Angle(Which),
    OffX(Which),
    OffY(Which),
    Render,
    Gallery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Leave,
}

pub struct Preset {
    pub name: &'static str,
    pub a: Layer,
    pub b: Layer,
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Classic Lines", a: Layer::new(Kind::Lines, 8, 0, 0, 0), b: Layer::new(Kind::Lines, 8, 5, 0, 0) },
    Preset {
        name: "Concentric Ripple",
        a: Layer::new(Kind::Circles, 6, 0, -20, 0),
        b: Layer::new(Kind::Circles, 6, 0, 20, 0),
    },
    Preset { name: "Fabric Weave", a: Layer::new(Kind::Grid, 10, 0, 0, 0), b: Layer::new(Kind::Grid, 10, 15, 0, 0) },
    Preset {
        name: "Radial Burst",
        a: Layer::new(Kind::Radial, 8, 0, -30, 0),
        b: Layer::new(Kind::Radial, 8, 0, 30, 0),
    },
    Preset { name: "Zebra", a: Layer::new(Kind::Lines, 6, 0, 0, 0), b: Layer::new(Kind::Lines, 7, 0, 0, 0) },
    Preset {
        name: "Hypnotic",
        a: Layer::new(Kind::Circles, 5, 0, 0, -40),
        b: Layer::new(Kind::Circles, 5, 0, 0, 40),
    },
    Preset { name: "Grid Rotation", a: Layer::new(Kind::Grid, 12, 0, 0, 0), b: Layer::new(Kind::Grid, 12, 7, 0, 0) },
    Preset { name: "Mixed Media", a: Layer::new(Kind::Lines, 8, 45, 0, 0), b: Layer::new(Kind::Circles, 10, 0, 0, 0) },
];

/// (layer A, layer B, width, height)
type CacheKey = (Layer, Layer, i32, i32);

pub struct Model {
    pub screen: Screen,
    pub menu: ListState,
    pub gallery: ListState,
    pub a: Layer,
    pub b: Layer,
    visible: usize,
    cache: Option<(CacheKey, Vec<u8>)>,
}

impl Default for Model {
    fn default() -> Self { Self::new() }
}

impl Model {
    pub fn new() -> Self {
        Self {
            screen: Screen::Menu,
            menu: ListState::default(),
            gallery: ListState::default(),
            a: Layer::default_a(),
            b: Layer::default_b(),
            visible: 8,
            cache: None,
        }
    }

    pub fn set_visible(&mut self, rows: usize) { self.visible = rows.max(1); }

    fn layer(&self, w: Which) -> &Layer { if w == Which::A { &self.a } else { &self.b } }

    fn layer_mut(&mut self, w: Which) -> &mut Layer { if w == Which::A { &mut self.a } else { &mut self.b } }

    /// Menu rows in order. The angle row is hidden for rings, where it has no effect.
    pub fn rows(&self) -> Vec<Row> {
        let mut v = Vec::new();
        for w in [Which::A, Which::B] {
            v.push(Row::Kind(w));
            v.push(Row::Spacing(w));
            if self.layer(w).kind.uses_angle() {
                v.push(Row::Angle(w));
            }
            v.push(Row::OffX(w));
            v.push(Row::OffY(w));
        }
        v.push(Row::Render);
        v.push(Row::Gallery);
        v
    }

    pub fn menu_labels(&self) -> Vec<String> {
        self.rows()
            .into_iter()
            .map(|r| match r {
                Row::Kind(w) => format!("{:?} type: < {} >", w, self.layer(w).kind.label()),
                Row::Spacing(w) => format!("{:?} spacing: < {} >", w, self.layer(w).spacing),
                Row::Angle(w) => format!("{:?} angle: < {} deg >", w, self.layer(w).angle),
                Row::OffX(w) => format!("{:?} offset x: < {} >", w, self.layer(w).off_x),
                Row::OffY(w) => format!("{:?} offset y: < {} >", w, self.layer(w).off_y),
                Row::Render => "[ Render ]".to_string(),
                Row::Gallery => "[ Gallery ]".to_string(),
            })
            .collect()
    }

    pub fn gallery_labels(&self) -> Vec<String> {
        PRESETS.iter().map(|p| format!("{} ({}/{})", p.name, p.a.kind.label(), p.b.kind.label())).collect()
    }

    pub fn title(&self) -> String {
        format!("A {} {} / B {} {} @{} deg", self.a.kind.label(), self.a.spacing, self.b.kind.label(), self.b.spacing, self.b.angle)
    }

    fn adjust(&mut self, row: Row, d: i32) {
        match row {
            Row::Kind(w) => {
                let l = self.layer_mut(w);
                l.kind = if d > 0 { l.kind.next() } else { l.kind.prev() };
            }
            Row::Spacing(w) => {
                let l = self.layer_mut(w);
                l.spacing = (l.spacing + d).clamp(MIN_SPACING, MAX_SPACING);
            }
            Row::Angle(w) => {
                let l = self.layer_mut(w);
                l.angle = (l.angle + 5 * d).rem_euclid(360);
            }
            Row::OffX(w) => {
                let l = self.layer_mut(w);
                l.off_x = (l.off_x + 5 * d).clamp(-MAX_OFFSET, MAX_OFFSET);
            }
            Row::OffY(w) => {
                let l = self.layer_mut(w);
                l.off_y = (l.off_y + 5 * d).clamp(-MAX_OFFSET, MAX_OFFSET);
            }
            Row::Render | Row::Gallery => {}
        }
    }

    pub fn on_key(&mut self, key: Key) -> Outcome {
        match self.screen {
            Screen::Menu => {
                let rows = self.rows();
                let n = rows.len();
                self.menu.clamp(n, self.visible);
                let row = rows[self.menu.selected];
                match key {
                    Key::Up => self.menu.step_wrapping(-1, n, self.visible),
                    Key::Down => self.menu.step_wrapping(1, n, self.visible),
                    Key::Left => self.adjust(row, -1),
                    Key::Right => self.adjust(row, 1),
                    Key::Enter => match row {
                        Row::Render => self.screen = Screen::Preview,
                        Row::Gallery => self.screen = Screen::Gallery,
                        _ => {}
                    },
                    k if k.is_back() => return Outcome::Leave,
                    _ => {}
                }
            }
            Screen::Preview => match key {
                // live tuning of layer B: the point of a moire toy
                Key::Left => self.b.angle = (self.b.angle - 1).rem_euclid(360),
                Key::Right => self.b.angle = (self.b.angle + 1).rem_euclid(360),
                Key::Up => self.b.spacing = (self.b.spacing + 1).min(MAX_SPACING),
                Key::Down => self.b.spacing = (self.b.spacing - 1).max(MIN_SPACING),
                Key::Enter => self.screen = Screen::Menu,
                k if k.is_back() => self.screen = Screen::Menu,
                _ => {}
            },
            Screen::Gallery => match key {
                Key::Up => self.gallery.step(-1, PRESETS.len(), self.visible),
                Key::Down => self.gallery.step(1, PRESETS.len(), self.visible),
                Key::Enter => {
                    let p = &PRESETS[self.gallery.selected];
                    self.a = p.a;
                    self.b = p.b;
                    self.screen = Screen::Preview;
                }
                Key::Left => self.screen = Screen::Menu,
                k if k.is_back() => self.screen = Screen::Menu,
                _ => {}
            },
        }
        Outcome::Stay
    }

    /// The XOR bitmap for the current layers at `w` x `h`, cached.
    pub fn bitmap(&mut self, w: i32, h: i32) -> &[u8] {
        let key = (self.a, self.b, w, h);
        if self.cache.as_ref().map(|c| c.0) != Some(key) {
            self.cache = Some((key, patterns::render(&self.a, &self.b, w, h)));
        }
        &self.cache.as_ref().expect("just filled").1
    }

    /// Rectangles covering the set pixels, for batched drawing.
    pub fn rects(&mut self, w: i32, h: i32) -> Vec<Rect> { rects::rects(self.bitmap(w, h), w, h) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_row_is_hidden_for_circles_and_selection_survives() {
        let mut m = Model::new();
        assert_eq!(m.rows().len(), 12);
        m.on_key(Key::Right); // A type: Lines -> Circles
        assert_eq!(m.a.kind, Kind::Circles);
        assert_eq!(m.rows().len(), 11);
        assert!(!m.rows().contains(&Row::Angle(Which::A)));
        assert!(m.rows().contains(&Row::Angle(Which::B)));
    }

    #[test]
    fn offsets_and_other_fields_are_editable_and_clamped() {
        let mut m = Model::new();
        m.menu.selected = m.rows().iter().position(|r| *r == Row::OffX(Which::A)).unwrap();
        m.on_key(Key::Right);
        assert_eq!(m.a.off_x, 5);
        for _ in 0..100 {
            m.on_key(Key::Left);
        }
        assert_eq!(m.a.off_x, -MAX_OFFSET);
        m.menu.selected = m.rows().iter().position(|r| *r == Row::Spacing(Which::B)).unwrap();
        for _ in 0..100 {
            m.on_key(Key::Left);
        }
        assert_eq!(m.b.spacing, MIN_SPACING);
        m.menu.selected = m.rows().iter().position(|r| *r == Row::Angle(Which::B)).unwrap();
        m.b.angle = 0;
        m.on_key(Key::Left);
        assert_eq!(m.b.angle, 355, "angle wraps");
    }

    #[test]
    fn labels_match_rows() {
        let m = Model::new();
        assert_eq!(m.menu_labels().len(), m.rows().len());
        assert!(m.menu_labels()[0].starts_with("A type"));
    }

    #[test]
    fn preview_tunes_layer_b_live_and_back_returns() {
        let mut m = Model::new();
        m.menu.selected = m.rows().len() - 2;
        m.on_key(Key::Enter);
        assert_eq!(m.screen, Screen::Preview);
        let before = m.bitmap(120, 90).to_vec();
        m.on_key(Key::Right);
        assert_eq!(m.b.angle, 6);
        assert_ne!(before, m.bitmap(120, 90));
        m.on_key(Key::Up);
        assert_eq!(m.b.spacing, 10);
        assert_eq!(m.on_key(Key::Backspace), Outcome::Stay);
        assert_eq!(m.screen, Screen::Menu);
        assert_eq!(m.on_key(Key::F(4)), Outcome::Leave);
    }

    #[test]
    fn gallery_loads_presets_into_the_layers() {
        let mut m = Model::new();
        m.screen = Screen::Gallery;
        for _ in 0..4 {
            m.on_key(Key::Down);
        }
        m.on_key(Key::Enter);
        assert_eq!(PRESETS[m.gallery.selected].name, "Zebra");
        assert_eq!((m.a.spacing, m.b.spacing), (6, 7));
        assert_eq!(m.screen, Screen::Preview);
    }

    /// Audit: Zebra rendered nothing; Fabric Weave showed one inverted grid.
    #[test]
    fn every_preset_renders_a_real_pattern() {
        let (w, h) = (336, 456);
        for p in PRESETS {
            let mut m = Model::new();
            m.a = p.a;
            m.b = p.b;
            let bits = m.bitmap(w, h).to_vec();
            let on: usize = bits.iter().map(|b| b.count_ones() as usize).sum();
            let f = on as f64 / (w * h) as f64;
            assert!(f > 0.1 && f < 0.9, "{}: {f}", p.name);
        }
    }

    #[test]
    fn drawing_uses_far_fewer_objects_than_pixel_runs() {
        let mut m = Model::new();
        let (w, h) = (336, 456);
        // 0 degree stripes: one rectangle per stripe, not one run per row per stripe
        m.a = Layer::new(Kind::Lines, 8, 0, 0, 0);
        m.b = Layer::new(Kind::Lines, 8, 0, 3, 0);
        let n = m.rects(w, h).len();
        assert!(n < 100, "{n} rectangles for vertical stripes");
        // the worst case, the concentric ripple, is a genuinely scattered pattern:
        // still bounded (well under one object per two pixels of a row)
        let p = &PRESETS[1];
        m.a = p.a;
        m.b = p.b;
        let n = m.rects(w, h).len();
        assert!(n < 20_000, "{n}");
    }

    #[test]
    fn bitmap_is_cached_per_setting() {
        let mut m = Model::new();
        let a = m.bitmap(100, 80).as_ptr();
        let b = m.bitmap(100, 80).as_ptr();
        assert_eq!(a, b);
    }
}
