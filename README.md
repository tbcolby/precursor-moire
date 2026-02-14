# Moiré Generator

An interference pattern generator for the [Precursor](https://precursor.dev) handheld, creating moiré effects by XOR-overlaying two configurable pattern layers on the 336×536 monochrome 1-bit display.

## Features

- **4 pattern types** — Lines, Circles, Radial, Grid
- **Two independent layers** — Each with type, spacing, and angle controls
- **XOR overlay** — Natural moiré interference from binary pattern combination
- **8 built-in presets** — Classic Lines, Concentric Ripple, Fabric Weave, and more
- **Per-pixel rendering** — Full bitmap computation with run-length optimized display

## Controls

| Key | Action |
|-----|--------|
| Up/Down | Navigate menu fields |
| Left/Right | Adjust parameters / cycle pattern types |
| Enter | Render pattern / select preset |
| Left/Menu | Return to menu from preview |
| Menu | Quit from main menu |

## Architecture

- `patterns.rs` — Layer rendering (Lines, Circles, Radial, Grid), integer trig, XOR bitmap overlay, octant-based angle sectors
- `app.rs` — 3-state machine (Menu → Preview → Gallery), 8 presets, dual-layer configuration
- `ui.rs` — Dual-layer parameter menu, run-length bitmap display, scrollable gallery
- `main.rs` — Standard Xous event loop

---

## Development

This app was developed using the methodology described in [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) — an LLM-assisted approach to Precursor app development on macOS ARM64.

---

## Author

Made by Tyler Colby — [Colby's Data Movers, LLC](https://colbysdatamovers.com)

Contact: [tyler@colbysdatamovers.com](mailto:tyler@colbysdatamovers.com) | [GitHub Issues](https://github.com/tbcolby/precursor-moire/issues)

---

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for the full text.
