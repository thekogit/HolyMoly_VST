# Build notes

- NIH-plug revision: de421011f41a6d10fc8c7a6084e4f4dee0143683
- egui version (from Cargo.lock): 0.31.1
- Build: `cargo xtask bundle holy_moly --release` → `target\bundled\Holy Moly.vst3`
- Check: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1`
- Licence: GPL-3.0-or-later (plan.md B11)
