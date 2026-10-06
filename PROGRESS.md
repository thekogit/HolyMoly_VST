# PROGRESS

Current step: S03

## Steps
```
[x] S00 Workspace setup
[x] S01 Cargo project and empty VST3 (SCAFFOLD)
[x] S02 Pin the NIH-plug revision
[ ] S03 fmt.rs — value text
[ ] S04 params.rs — the 52 parameters
[ ] S05 dsp/util.rs and dsp/testutil.rs
[ ] S06 dsp/oversample.rs
[ ] S07 dsp/drive.rs
[ ] S08 dsp/fuzz.rs
[ ] S09 dsp/ring.rs
[ ] S10 dsp/vibe.rs
[ ] S11 dsp/prism.rs
[ ] S12 dsp/echo.rs
[ ] S13 dsp/looper.rs
[ ] S14 dsp/tuner.rs
[ ] S15 shared.rs and chain.rs
[ ] S16 lib.rs final wiring, bundle, pluginval, HUMAN CHECK 1
[ ] S17 Fonts
[ ] S18 Editor skeleton and background
[ ] S19 paint.rs and layout.rs
[ ] S20 Knob widget
[ ] S21 Selector, toggle, push button, LED, footswitch, icons
[ ] S22 Top strip and tuner display
[ ] S23 Pedal frame and artwork
[ ] S24 Row 1 pedals (Drive, Fuzz, Ring, Vibe)
[ ] S25 Row 2 pedals (Prism, Echo, Looper) and loop display
[ ] S26 HUMAN CHECK 2 (visual) and polish
[ ] S27 Release checks and final acceptance
```

## Notes
<!-- one line per finished step: "S0x: what was done, test count" -->
- S00: git repo, folders, reference\holy-moly-editor.html. Concept .md NOT present in workspace (HUMAN_CHECKS #1). rustc 1.99 lives in %USERPROFILE%\.cargo\bin (prepended to PATH per command). Sandbox makes .git owned by Administrators → git runs with `-c safe.directory=*`.
- S01: scaffold builds, CHECK PASSED (0 tests), bundle created. nih-plug resolved to de421011, egui 0.31.1. Network steps (first build, bundle/cargo metadata) run outside the sandbox.
- S02: pinned rev de421011f41a6d10fc8c7a6084e4f4dee0143683 in all 3 git deps; egui 0.31.1; docs\API_NOTES.md (68 lines), docs\BUILD.md; Cargo.lock 4-line change; CHECK PASSED.

## Deviations
<!-- every minimal fix to a file from an earlier step: file, reason, change -->
- S01 `.cargo/config.toml`: clippy `-D warnings` failed on rustc 1.99 with `semicolon_in_expressions_from_non_local_macros` raised inside `nih_export_vst3!` (vst3_com::vtable, dependency code). Not in Part E → E-X1. Added `[build] rustflags = ["-A", "semicolon_in_expressions_from_non_local_macros"]` (kept out of lib.rs so it survives the VERBATIM lib.rs replacements).
