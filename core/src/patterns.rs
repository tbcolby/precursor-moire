//! Moire layers. Each layer is a 1-bit pattern; two layers are XORed into one
//! bitmap, where the interference shows up as slow beats.
//!
//! Spacing means one thing for every kind that has a period: the distance in
//! pixels between corresponding edges (lines, grid cells, rings), with a 50%
//! duty cycle. (The old code used a period of 2 x spacing at 25% duty for lines,
//! 2 x spacing for rings and 1 x spacing for grids.) Radial layers use
//! `4 * spacing` equal sectors, alternately on and off.
//!
//! All angles are whole degrees, trig is `fixed::sin/cos`. The old degree sine
//! was 0 at 0 and 90 degrees, so every line layer there was solid and the
//! "Zebra" preset XORed to a blank screen.

use precursor_appkit_core::fixed::{cos, sin, TURN};

use crate::rects::row_bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Parallel lines perpendicular to `angle`.
    Lines,
    /// Concentric rings.
    Circles,
    /// Spokes from the centre, rotated by `angle`.
    Radial,
    /// Two line sets at `angle` and `angle + 90`.
    Grid,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Lines => "Lines",
            Kind::Circles => "Circles",
            Kind::Radial => "Radial",
            Kind::Grid => "Grid",
        }
    }

    pub fn next(self) -> Kind {
        match self {
            Kind::Lines => Kind::Circles,
            Kind::Circles => Kind::Radial,
            Kind::Radial => Kind::Grid,
            Kind::Grid => Kind::Lines,
        }
    }

    pub fn prev(self) -> Kind {
        match self {
            Kind::Lines => Kind::Grid,
            Kind::Circles => Kind::Lines,
            Kind::Radial => Kind::Circles,
            Kind::Grid => Kind::Radial,
        }
    }

    /// Whether `angle` changes the picture (rings are rotation invariant).
    pub fn uses_angle(self) -> bool { self != Kind::Circles }
}

pub const MIN_SPACING: i32 = 3;
pub const MAX_SPACING: i32 = 30;
pub const MAX_OFFSET: i32 = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer {
    pub kind: Kind,
    /// Period in pixels (see module docs).
    pub spacing: i32,
    /// Whole degrees, 0..360.
    pub angle: i32,
    /// Shift of the pattern centre from the middle of the bitmap.
    pub off_x: i32,
    pub off_y: i32,
}

impl Layer {
    pub const fn new(kind: Kind, spacing: i32, angle: i32, off_x: i32, off_y: i32) -> Layer {
        Layer { kind, spacing, angle, off_x, off_y }
    }

    pub fn default_a() -> Layer { Layer::new(Kind::Lines, 8, 0, 0, 0) }

    pub fn default_b() -> Layer { Layer::new(Kind::Lines, 9, 5, 0, 0) }
}

/// Angle in 4096-unit turns for whole degrees.
fn deg_units(deg: i32) -> i32 { (deg.rem_euclid(360) as i64 * TURN as i64 / 360) as i32 }

/// Angle of `(dx, dy)` in 8192 units per turn (screen coordinates, clockwise),
/// by octant with a quadratic atan correction (error under 0.25 degree).
pub fn angle8192(dx: i32, dy: i32) -> i32 {
    if dx == 0 && dy == 0 {
        return 0;
    }
    let (ax, ay) = (dx.abs(), dy.abs());
    // t in 0..=1024: the smaller over the larger coordinate
    let (t, steep) = if ax >= ay { (ay * 1024 / ax, false) } else { (ax * 1024 / ay, true) };
    // atan(t) in 8192-unit angle: t * (1024 + 356 * (1 - t)), t in Q10
    let a = (t * (1024 + 356 * (1024 - t) / 1024)) / 1024; // 0..=1024 (45 degrees)
    // fold the octant back into 0..8192
    let first = if steep { 2048 - a } else { a }; // angle of (|dx|, |dy|) in the first quadrant
    match (dx >= 0, dy >= 0) {
        (true, true) => first,
        (false, true) => 4096 - first,
        (false, false) => 4096 + first,
        (true, false) => 8192 - first,
    }
    .rem_euclid(8192)
}

/// Toggle (XOR) every pixel of `bits` (row-padded, MSB first) where `layer` is on.
pub fn toggle_layer(layer: &Layer, bits: &mut [u8], w: i32, h: i32) {
    let rb = row_bytes(w);
    let cx = w / 2 + layer.off_x;
    let cy = h / 2 + layer.off_y;
    let flip = |bits: &mut [u8], x: i32, y: i32| bits[y as usize * rb + x as usize / 8] ^= 0x80 >> (x as usize % 8);
    let sp = layer.spacing.clamp(MIN_SPACING, MAX_SPACING);
    match layer.kind {
        Kind::Lines | Kind::Grid => {
            let a = deg_units(layer.angle);
            let (c, s) = (cos(a), sin(a));
            let period = sp << 16;
            let half = period / 2;
            for y in 0..h {
                let dy = y - cy;
                // projection on the line normal and (for grids) the second normal
                let mut p1 = dy * s - cx * c;
                let mut p2 = dy * c + cx * s;
                for x in 0..w {
                    let on1 = p1.rem_euclid(period) < half;
                    let on = on1 || (layer.kind == Kind::Grid && p2.rem_euclid(period) < half);
                    if on {
                        flip(bits, x, y);
                    }
                    p1 += c;
                    p2 -= s;
                }
            }
        }
        Kind::Circles => {
            // d4 = floor(4 * distance), kept incrementally along each row
            let span = 4 * sp;
            let maxdx = cx.max(w - 1 - cx).max(0);
            let mut ring = vec![false; maxdx as usize + 1];
            for y in 0..h {
                let dy = (y - cy) as i64;
                let mut k: i64 = 0;
                for (dx, r) in ring.iter_mut().enumerate() {
                    let v = 16 * (dx as i64 * dx as i64 + dy * dy);
                    while (k + 1) * (k + 1) <= v {
                        k += 1;
                    }
                    *r = k % (span as i64) < (span / 2) as i64;
                }
                for x in 0..w {
                    if ring[(x - cx).unsigned_abs() as usize] {
                        flip(bits, x, y);
                    }
                }
            }
        }
        Kind::Radial => {
            let sectors = (sp * 4) as i64;
            let rot = (layer.angle.rem_euclid(360) as i64 * 8192 / 360) as i32;
            for y in 0..h {
                for x in 0..w {
                    let (dx, dy) = (x - cx, y - cy);
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let a = (angle8192(dx, dy) + rot).rem_euclid(8192) as i64;
                    if (a * sectors / 8192) % 2 == 0 {
                        flip(bits, x, y);
                    }
                }
            }
        }
    }
}

/// One layer as a bitmap.
pub fn render_layer(layer: &Layer, w: i32, h: i32) -> Vec<u8> {
    let mut bits = vec![0u8; row_bytes(w) * h.max(0) as usize];
    toggle_layer(layer, &mut bits, w, h);
    bits
}

/// Two layers XORed into one bitmap (one buffer, no intermediates).
pub fn render(a: &Layer, b: &Layer, w: i32, h: i32) -> Vec<u8> {
    let mut bits = vec![0u8; row_bytes(w) * h.max(0) as usize];
    toggle_layer(a, &mut bits, w, h);
    toggle_layer(b, &mut bits, w, h);
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rects::get;

    const W: i32 = 336;
    const H: i32 = 456;

    fn on_count(bits: &[u8]) -> usize { bits.iter().map(|b| b.count_ones() as usize).sum() }

    fn frac(bits: &[u8]) -> f64 { on_count(bits) as f64 / (W * H) as f64 }

    fn lines(spacing: i32, angle: i32) -> Layer { Layer::new(Kind::Lines, spacing, angle, 0, 0) }

    /// Audit: any Lines layer at 0 or 90 degrees had every pixel set (153,216 of 153,216).
    #[test]
    fn axis_aligned_lines_are_half_on_stripes() {
        for angle in [0, 90, 180, 270] {
            let b = render_layer(&lines(8, angle), W, H);
            let f = frac(&b);
            assert!((f - 0.5).abs() < 0.02, "angle {angle}: {f}");
        }
        // 0 degrees: vertical stripes, every row identical
        let b = render_layer(&lines(8, 0), W, H);
        for y in 1..H {
            for x in 0..W {
                assert_eq!(get(&b, W, x, y), get(&b, W, x, 0));
            }
        }
        // 90 degrees: horizontal stripes, every column identical
        let b = render_layer(&lines(8, 90), W, H);
        for x in 1..W {
            for y in 0..H {
                assert_eq!(get(&b, W, x, y), get(&b, W, 0, y));
            }
        }
    }

    #[test]
    fn stripe_period_equals_spacing_exactly() {
        for sp in [3, 6, 7, 10, 25] {
            let b = render_layer(&lines(sp, 0), W, H);
            for x in 0..W - sp {
                assert_eq!(get(&b, W, x, 10), get(&b, W, x + sp, 10), "spacing {sp} at {x}");
            }
            let run = (0..W).take_while(|&x| get(&b, W, (W / 2 + x).min(W - 1), 0) == get(&b, W, W / 2, 0)).count();
            assert!(run as i32 <= sp / 2 + 1, "spacing {sp}: run {run}");
        }
    }

    #[test]
    fn diagonal_lines_are_diagonal() {
        let b = render_layer(&lines(8, 45), W, H);
        // constant along the other diagonal (x + y changes nothing for angle 45's normal (1,1)/sqrt2)
        let mut same = 0;
        let mut total = 0;
        for y in 20..H - 20 {
            for x in 20..W - 20 {
                total += 1;
                if get(&b, W, x, y) == get(&b, W, x + 5, y - 5) {
                    same += 1;
                }
            }
        }
        assert!(same as f64 / total as f64 > 0.97, "{same}/{total}");
        assert!((frac(&b) - 0.5).abs() < 0.02);
    }

    /// Audit: the Zebra preset (both layers at 0 degrees, spacing 6 and 7) XORed to nothing.
    #[test]
    fn zebra_is_a_visible_beat_with_period_42() {
        let (a, b) = (lines(6, 0), lines(7, 0));
        let x = render(&a, &b, W, H);
        let f = frac(&x);
        assert!(f > 0.3 && f < 0.7, "{f}");
        for px in 0..W - 42 {
            assert_eq!(get(&x, W, px, 5), get(&x, W, px + 42, 5), "beat period at {px}");
        }
        // and it is not periodic at a smaller scale
        assert!((0..W - 21).any(|px| get(&x, W, px, 5) != get(&x, W, px + 21, 5)));
    }

    #[test]
    fn identical_layers_cancel() {
        for kind in [Kind::Lines, Kind::Circles, Kind::Radial, Kind::Grid] {
            let l = Layer::new(kind, 7, 30, 11, -9);
            assert_eq!(on_count(&render(&l, &l, W, H)), 0, "{kind:?}");
        }
    }

    #[test]
    fn grid_is_three_quarters_on_at_every_angle() {
        for angle in [0, 15, 45, 90] {
            let b = render_layer(&Layer::new(Kind::Grid, 10, angle, 0, 0), W, H);
            let f = frac(&b);
            assert!((f - 0.75).abs() < 0.03, "angle {angle}: {f}");
        }
    }

    /// Audit: the octagonal distance estimate; rings must match the true distance.
    #[test]
    fn rings_match_true_euclidean_distance() {
        let sp = 6;
        let l = Layer::new(Kind::Circles, sp, 0, -20, 10);
        let b = render_layer(&l, W, H);
        let (cx, cy) = (W / 2 - 20, H / 2 + 10);
        let mut bad = 0;
        for y in 0..H {
            for x in 0..W {
                let d = (((x - cx).pow(2) + (y - cy).pow(2)) as f64).sqrt();
                let q = (d * 4.0 + 1e-9).floor() as i64;
                let want = q % (4 * sp as i64) < 2 * sp as i64;
                if get(&b, W, x, y) != want {
                    bad += 1;
                }
            }
        }
        // only float rounding right at ring edges may differ
        assert!(bad < 40, "{bad} pixels differ from the true circle");
        assert!((frac(&b) - 0.5).abs() < 0.03);
    }

    #[test]
    fn rings_are_round_not_octagonal() {
        let l = Layer::new(Kind::Circles, 10, 0, 0, 0);
        let b = render_layer(&l, 300, 300);
        // first ring edge along the axis and along the diagonal, in distance terms
        let edge = |step: (i32, i32)| {
            let start = get(&b, 300, 150, 150);
            (1..140)
                .find(|&t| get(&b, 300, 150 + step.0 * t, 150 + step.1 * t) != start)
                .map(|t| ((step.0 * t).pow(2) as f64 + (step.1 * t).pow(2) as f64).sqrt())
                .unwrap()
        };
        let (axis, diag) = (edge((1, 0)), edge((1, 1)));
        assert!((axis - diag).abs() < 1.5, "axis {axis} vs diagonal {diag}");
    }

    #[test]
    fn atan_is_accurate_in_every_octant() {
        for deg in 0..360 {
            let rad = (deg as f64).to_radians();
            let (dx, dy) = ((1000.0 * rad.cos()).round() as i32, (1000.0 * rad.sin()).round() as i32);
            let want = deg as f64 * 8192.0 / 360.0;
            let got = angle8192(dx, dy) as f64;
            let diff = ((got - want + 4096.0).rem_euclid(8192.0) - 4096.0).abs();
            assert!(diff < 8.0, "{deg} deg: {got} vs {want}");
        }
    }

    #[test]
    fn radial_has_the_requested_number_of_spokes() {
        for sp in [3, 5, 8] {
            let l = Layer::new(Kind::Radial, sp, 0, 0, 0);
            let b = render_layer(&l, 300, 300);
            let mut transitions = 0;
            let mut prev = None;
            for i in 0..3600 {
                let a = (i as f64 / 3600.0) * std::f64::consts::TAU;
                let (x, y) = (150 + (100.0 * a.cos()).round() as i32, 150 + (100.0 * a.sin()).round() as i32);
                let v = get(&b, 300, x, y);
                if prev.is_some_and(|p| p != v) {
                    transitions += 1;
                }
                prev = Some(v);
            }
            // sectors = 4 * spacing; the wrap-around adds or hides one transition
            let sectors = 4 * sp;
            assert!((transitions - sectors).abs() <= 1, "spacing {sp}: {transitions} transitions");
        }
    }

    #[test]
    fn radial_rotation_follows_the_angle() {
        // 36 sectors of 10 degrees; rotating by two sectors maps the pattern onto itself
        let a = render_layer(&Layer::new(Kind::Radial, 9, 0, 0, 0), 200, 200);
        let b = render_layer(&Layer::new(Kind::Radial, 9, 20, 0, 0), 200, 200);
        let c = render_layer(&Layer::new(Kind::Radial, 9, 10, 0, 0), 200, 200);
        let diff = |p: &[u8], q: &[u8]| p.iter().zip(q).map(|(x, y)| (x ^ y).count_ones()).sum::<u32>() as f64 / 40000.0;
        assert!(diff(&a, &b) < 0.03, "{}", diff(&a, &b));
        assert!(diff(&a, &c) > 0.9, "one sector shifts on/off: {}", diff(&a, &c));
    }

    #[test]
    fn offsets_move_the_pattern() {
        let a = render_layer(&Layer::new(Kind::Circles, 6, 0, 0, 0), W, H);
        let b = render_layer(&Layer::new(Kind::Circles, 6, 0, 30, 0), W, H);
        assert_ne!(a, b);
        // the shifted rings are centred 30 px to the right
        let (cx, cy) = (W / 2 + 30, H / 2);
        assert_eq!(get(&b, W, cx, cy), get(&a, W, W / 2, H / 2));
        let l = render_layer(&Layer::new(Kind::Lines, 8, 0, 0, 0), W, H);
        let m = render_layer(&Layer::new(Kind::Lines, 8, 0, 3, 0), W, H);
        for x in 0..W - 3 {
            assert_eq!(get(&l, W, x, 0), get(&m, W, x + 3, 0));
        }
    }

    #[test]
    fn rendering_is_deterministic_and_handles_odd_sizes() {
        let (a, b) = (Layer::default_a(), Layer::default_b());
        assert_eq!(render(&a, &b, 101, 77), render(&a, &b, 101, 77));
        assert_eq!(render(&a, &b, 101, 77).len(), 13 * 77);
    }
}
