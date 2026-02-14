//! Moiré pattern generation.
//!
//! Generates interference patterns by overlaying two grids of lines or circles.
//! On a 1-bit display, moiré emerges naturally from XOR overlap.

extern crate alloc;
use alloc::vec::Vec;

/// Pattern type for each layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternKind {
    Lines,          // Parallel lines at an angle
    Circles,        // Concentric circles
    Radial,         // Lines radiating from center
    Grid,           // Crosshatch grid
}

impl PatternKind {
    pub fn label(&self) -> &'static str {
        match self {
            PatternKind::Lines => "Lines",
            PatternKind::Circles => "Circles",
            PatternKind::Radial => "Radial",
            PatternKind::Grid => "Grid",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PatternKind::Lines => PatternKind::Circles,
            PatternKind::Circles => PatternKind::Radial,
            PatternKind::Radial => PatternKind::Grid,
            PatternKind::Grid => PatternKind::Lines,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            PatternKind::Lines => PatternKind::Grid,
            PatternKind::Circles => PatternKind::Lines,
            PatternKind::Radial => PatternKind::Circles,
            PatternKind::Grid => PatternKind::Radial,
        }
    }
}

/// Parameters for a moiré layer.
#[derive(Debug, Clone, Copy)]
pub struct Layer {
    pub kind: PatternKind,
    pub spacing: i32,     // pixels between lines/circles
    pub angle: i32,       // rotation in degrees (0-359), for Lines/Grid
    pub offset_x: i32,    // center offset X
    pub offset_y: i32,    // center offset Y
}

impl Layer {
    pub fn default_a() -> Self {
        Self {
            kind: PatternKind::Lines,
            spacing: 8,
            angle: 0,
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn default_b() -> Self {
        Self {
            kind: PatternKind::Lines,
            spacing: 9,
            angle: 5,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

/// Fast integer sine (same approach as spirograph).
/// Input: angle in degrees (0-359).
/// Output: sin * 1024 (fixed-point 10-bit).
fn sin_deg(deg: i32) -> i32 {
    // Convert degrees to 1024-based angle: deg * 1024 / 360
    let a = ((deg % 360 + 360) % 360) as i64;
    let angle = (a * 1024 / 360) as i32;

    let norm = ((angle % 1024) + 1024) % 1024;

    let (idx, sign) = if norm < 256 {
        (norm, 1i32)
    } else if norm < 512 {
        (512 - norm, 1)
    } else if norm < 768 {
        (norm - 512, -1)
    } else {
        (1024 - norm, -1)
    };

    let y = (4 * idx * (256 - idx)) as i64;
    let raw = (y * 1024 / (256 * 256) as i64) as i32;
    raw * sign
}

fn cos_deg(deg: i32) -> i32 {
    sin_deg(deg + 90)
}

/// Generate a 1-bit bitmap for a single layer.
/// Returns a Vec<u8> where each bit represents a pixel (row-major, MSB first).
/// Width must be a multiple of 8 for simplicity.
pub fn render_layer(layer: &Layer, width: i32, height: i32) -> Vec<u8> {
    let row_bytes = (width / 8) as usize;
    let mut bitmap = alloc::vec![0u8; row_bytes * height as usize];

    let cx = width / 2 + layer.offset_x;
    let cy = height / 2 + layer.offset_y;

    match layer.kind {
        PatternKind::Lines => {
            // Parallel lines perpendicular to angle direction
            let s = sin_deg(layer.angle);
            let c = cos_deg(layer.angle);
            for py in 0..height {
                for px in 0..width {
                    // Project point onto angle direction
                    let dx = px - cx;
                    let dy = py - cy;
                    // Distance along angle direction: dx*cos + dy*sin (in *1024)
                    let proj = dx * c + dy * s;
                    // Quantize by spacing
                    let dist = ((proj / (layer.spacing * 1024)).abs()) % 2;
                    if dist == 0 {
                        // half lines on, half off
                        let within = (proj.abs() / 1024) % layer.spacing;
                        if within < layer.spacing / 2 {
                            let idx = (py * width + px) as usize;
                            bitmap[idx / 8] |= 0x80 >> (idx % 8);
                        }
                    }
                }
            }
        }
        PatternKind::Circles => {
            for py in 0..height {
                for px in 0..width {
                    let dx = px - cx;
                    let dy = py - cy;
                    // Integer distance approximation: max(|dx|,|dy|) + min(|dx|,|dy|)*3/8
                    let ax = dx.abs();
                    let ay = dy.abs();
                    let dist = if ax > ay {
                        ax + ay * 3 / 8
                    } else {
                        ay + ax * 3 / 8
                    };
                    if (dist / layer.spacing) % 2 == 0 {
                        let idx = (py * width + px) as usize;
                        bitmap[idx / 8] |= 0x80 >> (idx % 8);
                    }
                }
            }
        }
        PatternKind::Radial => {
            for py in 0..height {
                for px in 0..width {
                    let dx = px - cx;
                    let dy = py - cy;
                    if dx == 0 && dy == 0 { continue; }
                    // Approximate angle via octant-based atan2
                    // Use sector counting: divide circle into spacing sectors
                    let sectors = layer.spacing * 4; // total sectors
                    // atan2 approximation: sector = atan2(dy,dx) * sectors / (2*pi)
                    // Use integer atan2: based on octant
                    let angle_sector = fast_angle_sector(dx, dy, sectors);
                    if angle_sector % 2 == 0 {
                        let idx = (py * width + px) as usize;
                        bitmap[idx / 8] |= 0x80 >> (idx % 8);
                    }
                }
            }
        }
        PatternKind::Grid => {
            // Crosshatch: lines at angle and angle+90
            let s1 = sin_deg(layer.angle);
            let c1 = cos_deg(layer.angle);
            let s2 = sin_deg(layer.angle + 90);
            let c2 = cos_deg(layer.angle + 90);
            for py in 0..height {
                for px in 0..width {
                    let dx = px - cx;
                    let dy = py - cy;
                    let proj1 = (dx * c1 + dy * s1).abs() / 1024;
                    let proj2 = (dx * c2 + dy * s2).abs() / 1024;
                    let on1 = (proj1 % layer.spacing) < layer.spacing / 2;
                    let on2 = (proj2 % layer.spacing) < layer.spacing / 2;
                    if on1 || on2 {
                        let idx = (py * width + px) as usize;
                        bitmap[idx / 8] |= 0x80 >> (idx % 8);
                    }
                }
            }
        }
    }

    bitmap
}

/// Fast integer angle sector (0..sectors-1) from dx, dy.
fn fast_angle_sector(dx: i32, dy: i32, sectors: i32) -> i32 {
    if sectors <= 0 { return 0; }
    let ax = dx.abs();
    let ay = dy.abs();
    // Octant-based: compute fractional angle within octant
    let octant;
    let ratio; // 0..1024 within octant
    if ax >= ay {
        if ax == 0 { return 0; }
        ratio = (ay * 1024 / ax).min(1024);
        octant = if dx >= 0 {
            if dy >= 0 { 0 } else { 7 }
        } else {
            if dy >= 0 { 3 } else { 4 }
        };
    } else {
        ratio = (ax * 1024 / ay).min(1024);
        octant = if dx >= 0 {
            if dy >= 0 { 1 } else { 6 }
        } else {
            if dy >= 0 { 2 } else { 5 }
        };
    }

    // Map octant + ratio to sector
    let angle_8192 = octant * 1024 + if octant % 2 == 0 { ratio } else { 1024 - ratio };
    // Total range: 0..8192 maps to 0..sectors
    ((angle_8192 as i64 * sectors as i64 / 8192) as i32).max(0).min(sectors - 1)
}

/// XOR two bitmaps together to create moiré interference.
pub fn xor_bitmaps(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}
