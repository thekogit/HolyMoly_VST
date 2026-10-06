# Holy Moly

**Holy Moly** is a stereo pedalboard effect plugin (VST3 and standalone) built in Rust using [NIH-plug](https://github.com/robbert-vdh/nih-plug) and [egui](https://github.com/emilk/egui).

It features a fixed series chain of seven guitar effects and utility processors designed with custom zero-allocation DSP and a custom hand-painted vector GUI:

1. **Top Strip**: Master Input/Output levels, Quality oversampling selector (1×/2×/4× FIR), Mute switch, and an integrated chromatic Tuner with 21-segment cents display powered by the YIN pitch-tracking algorithm.
2. **02. Drive**: Asymmetric clipping overdrive with 3-way low-frequency voicing (`Tight`, `Classic`, `Loose`).
3. **03. Fuzz**: Transistor shaper with variable bias gating and high-cut tone.
4. **04. Ring**: Dual-mode ring modulator (`Self` and `Osc` sine carrier) with shaper distortion.
5. **05. Vibe**: 4-stage optical phase shifter with `Chorus` and `Vibrato` modes.
6. **06. Prism**: Granular dual-stream pitch shifter with halo octave feedback.
7. **07. Echo**: Multi-algorithm delay (`Digital`, `Tape`, `Multi`, `Reverse`) with tempo sync, ping-pong, and trails.
8. **08. Looper**: 30-second stereo sound-on-sound phrase looper with reverse playback and MIDI control (notes 60/61/62).

---

## Building from Source

### Prerequisites
- Rust 1.80+ (tested on rustc 1.99 Windows MSVC)
- Windows PowerShell

### Build Commands
From the project root:

```powershell
# Build and bundle the VST3 plugin and standalone executable
cargo xtask bundle holy_moly --release
```

Outputs will be generated at:
- `target\bundled\Holy Moly.vst3`
- `target\bundled\Holy Moly.exe`

### Running Tests and Verification
To run the automated test suite, format check, clippy linter, and placeholder scan:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1
```

---

## Documentation
- [User Guide](docs/USER_GUIDE.md): Installation instructions, module descriptions, and MIDI control notes.
- [Visual Review](docs/VISUAL_REVIEW.md): Detailed visual inspection against the reference design.
- [Build Notes](docs/BUILD.md): Pinned dependency revisions and build notes.

---

## Licence
This project is licensed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later). See [LICENSE](LICENSE) for details.
Embedded fonts (`Martian Mono` and `Big Shoulders Stencil`) are licensed under the SIL Open Font License (OFL).
