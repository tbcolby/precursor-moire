# Moire Generator

An interference pattern toy for the [Precursor](https://precursor.dev) handheld: two configurable layers are XORed on the 1-bit display, and the moire shows up as slow beats between them.

Version 0.2. Not yet run on Renode or hardware: `capture.py` is written but unrun.

## Features

- Four layer kinds: Lines, Circles (rings), Radial (spokes) and Grid (two line sets at right angles).
- Spacing means the same thing for every kind with a period: the distance in pixels between corresponding edges, 50% on and 50% off (lines, rings and grid cells). Radial uses `4 x spacing` equal sectors.
- Per layer: kind, spacing (3 to 30), angle (Lines, Radial, Grid; hidden for rings, which it does not affect), and centre offsets x and y (up to 150 px), which drive presets such as "Hypnotic".
- Eight presets in a gallery: Classic Lines, Concentric Ripple, Fabric Weave, Radial Burst, Zebra, Hypnotic, Grid Rotation, Mixed Media.
- Live tuning in the preview: Left/Right turn layer B by 1 degree, Up/Down change its spacing, and the picture is redrawn.
- Rendering is integer maths only: `fixed::sin/cos` for line normals, an incremental integer square root for rings, an octant atan with a quadratic correction (under 0.25 degrees) for spokes. Both layers are XORed into one bitmap.
- Drawing merges each run of pixels with the identical run above it and sends rectangles to GAM in batches of 32 (`Screen::fill_rects`).

## Keys

Every screen leaves with Backspace or F4 (the `∴` menu key belongs to the OS).

| Screen | Keys |
|--------|------|
| Menu | Up/Down choose a field, Left/Right change it (angle and offsets in steps of 5), Enter on Render or Gallery, Backspace/F4 leave the app |
| Preview | Left/Right layer B angle, Up/Down layer B spacing, Enter/Backspace/F4 back to the menu |
| Gallery | Up/Down browse, Enter show, Left/Backspace/F4 back |

## Storage

Nothing is saved.

## Layout

```
core/          moire-core: layers, rectangle merging, state machine (24 tests)
src/main.rs    registration and message routing (no pump: nothing animates)
src/app.rs     keys -> core, drawing
```

Building: `bin/integrate --xous-root <xous-core> --grade <this repo>` from [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit). Host tests: `cd core && cargo test`. Manifest: `manifest-fragment.json`, key `moire` (GAM app name `gam::APP_NAME_MOIRE`).

## Fixed from the 0.1 audit

- The shared degree sine was 0 at 0 and 90 degrees, so every line or grid layer at those angles was solid and the "Zebra" preset XORed to a blank screen. Tests now check stripes are 50% on at 0, 90, 180 and 270 degrees, the stripe period equals the spacing exactly, diagonals are diagonal, grids are 75% on, and Zebra (spacing 6 and 7 at 0 degrees) is a visible beat that repeats every 42 px.
- Spacing meant a period of 2 x spacing at 25% duty for lines (with a double-width band at the centre), 2 x spacing for rings and 1 x spacing for grids: one definition now, tested.
- Rings used the octagonal `max + 3/8 min` distance estimate (up to 7% error): they use the true Euclidean distance, tested pixel for pixel against `f64` and for equal axis and diagonal ring radii.
- The display drew one `draw_line` per horizontal run (15,436 for Concentric Ripple): runs are merged vertically into rectangles and batched, so vertical stripes are one rectangle per stripe.
- Two layer bitmaps plus the XOR result were allocated on each render: one buffer, XORed in place; the result is cached until a setting changes.
- Layer offsets existed but could not be edited, and the angle row showed for rings: offsets are menu fields, and the angle row is hidden for rings.
- Radial layers can now be rotated with the angle.
- Unused imports, Apple PUA arrows, `∴` as quit, `terminate_process`, `UxType::Chat`, hard-coded 336x536 layout: replaced by appkit keys, `relinquish_focus`, a framebuffer app sized from the canvas.
- The preview only exited: it now tunes layer B live.

## Limitations

- Scattered patterns are still expensive to draw: Concentric Ripple needs about 19,000 rectangles, roughly 600 GAM messages, so expect a redraw of a few seconds. Vertical or horizontal stripes need only dozens. A bitmap blit (GAM's `ditherpunk` tiles) would remove this but is not enabled in the build.
- Each key press in the preview recomputes and redraws the full picture; there is no animation.
- Radial spokes use an approximate atan (error under 0.25 degrees), so sector edges are not exact.
- Odd spacings give an on/off split of exactly half a period but not whole pixels, so edges alternate by a pixel.

## Author

Made by Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com). Apache-2.0, see [LICENSE](LICENSE).
