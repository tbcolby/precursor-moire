# Moire Generator: agent notes

- Package `moire`; GAM app name `gam::APP_NAME_MOIRE` (manifest key `moire`, context name "Moire Generator"); xous-names server `_Moire Generator_`.
- `core/` (`moire-core`) holds `patterns` (layers XORed into one row-padded bitmap), `rects` (bitmap to merged rectangles) and the menu/preview/gallery `Model`, with no Xous dependencies. Run `cd core && cargo test`.
- Trig is `precursor_appkit_core::fixed`; never add a local sine table. No float maths outside tests.
- `src/` is the shell: `UxType::Framebuffer`, raw keys, no pump, bitmap drawn with `Screen::fill_rects`.
- Gate: `xous-dev-toolkit/bin/integrate --xous-root <tree> --grade <repo>`.
