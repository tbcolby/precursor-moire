# Moiré Generator — Build Notes

## What This Is
Interference pattern generator for Precursor. Overlays two configurable pattern layers via XOR to create moiré effects on 1-bit display.

## Key Decisions
- **Bitmap-based rendering**: Full pixel-by-pixel computation into Vec<u8> bitmaps, then XOR overlay. Necessary because moiré requires per-pixel evaluation.
- **Run-length display**: Horizontal runs of set pixels drawn as single line segments for efficiency instead of pixel-by-pixel GAM calls.
- **4 pattern types**: Lines (parallel at angle), Circles (concentric), Radial (spokes from center), Grid (crosshatch). All use integer math only.
- **Octant-based atan2**: Fast integer angle sector computation for Radial pattern without floating-point atan2.
- **Integer trig**: sin/cos in degrees using 1024-based parabolic approximation, consistent with other Wave 4 apps.

## Architecture
- 3 states: Menu, Preview, Gallery
- 8 presets showing diverse pattern combinations
- patterns.rs is self-contained, no Xous deps — testable independently
