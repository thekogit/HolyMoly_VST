# HOLY MOLY — VST3 BUILD MANUAL

| | |
|---|---|
| Document | HM-PLAN-002, Revision A, 06 OCT 2026 |
| Edition | Single-agent edition (one executor, sequential steps) |
| Product | Holy Moly stereo pedalboard effect, VST3, Rust + NIH-plug |
| Visual reference | `reference/holy-moly-editor.html` |
| Background reference | `reference/Holy Moly — VST3 Plugin Concept.md` |

This manual is complete. The executor must not need anything that is not written here, except the three reference files and the crates listed in Step S01.

---

# PART A — OPERATING RULES (READ AT THE START OF EVERY SESSION)

## A1. Word usage

| Word | Meaning |
|---|---|
| SHALL / SHALL NOT | Mandatory. No exceptions. |
| VERBATIM | Copy the code block character for character. Do not rename, reorder, "improve" or shorten it. |
| SPEC | Write code that implements the description exactly: every number, formula, colour and coordinate. |
| SCAFFOLD | Temporary code that a later step replaces. This is the only kind of temporary code allowed. |
| VERIFY | Commands to run, followed by the EXPECTED result. |
| OPTIONAL | Polish. If it fails after 2 attempts, skip it and log `skipped` in PROGRESS.md. |
| HUMAN CHECK | Something only the owner can do (for example, load the plugin in a DAW). Write it into `HUMAN_CHECKS.md` and continue. Do not wait. |

## A2. Advisory notices

> **WARNING** — Could cause loud or broken audio for the end user, or a crashed DAW.

> **CAUTION** — Could break the build or waste work.

> **NOTE** — Important information.

## A3. The work loop (repeat for every step)

```
STEP LOOP
1. PROGRESS.md ................................. OPEN, find first step not [x]
2. plan.md ..................................... READ that step fully, from its
                                                 heading to the next step heading
3. Referenced sections (Part C, Part E) ........ READ
4. Files ....................................... CREATE/EDIT only those listed
5. VERBATIM blocks ............................. COPY exactly
6. SPEC blocks ................................. IMPLEMENT exactly
7. VERIFY commands ............................. RUN, compare with EXPECTED
8. Failure ..................................... READ error, CHECK Part E, FIX, re-run
                                                 (max 5 attempts, then mark [!] and STOP)
9. Placeholder check (A5) ...................... PASS
10. git commit ................................. MESSAGE as given in the step
11. PROGRESS.md ................................ TICK [x], add one-line note
12. Next step .................................. START IMMEDIATELY (do not ask)
```

## A4. Absolute rules

1. Tests given in this manual are the specification. You SHALL NOT edit, delete, `#[ignore]` or loosen them. If a given test fails, the implementation is wrong. Fix the implementation.
2. You SHALL NOT write `todo!()`, `unimplemented!()`, `TODO`, `FIXME`, "simplified version", pass-through DSP, or a placeholder in place of an algorithm.
3. You SHALL NOT change any parameter ID in Table C1. Hosts save projects by these IDs.
4. You SHALL NOT add any crate that is not listed in Step S01.
5. You SHALL NOT write NIH-plug or egui API calls from memory when this manual gives the code. If the compiler disagrees with the manual's code, follow Part E. Never guess.
6. You SHALL NOT modify files from completed steps. Exceptions: the current step says so, or a compile error forces a minimal fix. Log every such fix in PROGRESS.md.
7. The shell is Windows PowerShell. Quote every path that contains spaces.
8. The audio thread (`process()` and everything it calls) SHALL NOT allocate, lock, print, or do file I/O. Allocation is allowed only in functions named `allocate` or `prepare`, and in `Default`/`new`.
9. Keep chat replies to the owner short. Details go in PROGRESS.md.

## A5. Placeholder check (run before every commit)

```powershell
Get-ChildItem -Recurse -Path src -Filter *.rs | Select-String -Pattern 'todo!\(|unimplemented!\(|TODO|FIXME|SCAFFOLD' | Select-Object -First 20
```

EXPECTED: no output.

Exception: before Step S16, lines containing `SCAFFOLD` in `src/lib.rs` are allowed.

## A6. Standard verification commands

| Name | Command | EXPECTED |
|---|---|---|
| TEST | `cargo test --lib` | last lines contain `test result: ok.` and `0 failed` |
| CLIPPY | `cargo clippy --lib -- -D warnings` | exits with code 0 |
| BUNDLE | `cargo xtask bundle holy_moly --release` | folder `target\bundled\Holy Moly.vst3` exists |

> **CAUTION** — BUNDLE fails with "access denied" if a DAW has the plugin loaded. Ask the owner to close the DAW (write it to HUMAN_CHECKS.md) and continue with the next step that does not need BUNDLE.

---

# PART B — FIXED DECISIONS

Do not reconsider these decisions. They exist so that nothing is left to judgement.

| # | Decision |
|---|---|
| B1 | **One crate** named `holy_moly` (cdylib + lib), plus the standard `xtask` crate. All DSP is in `src/dsp/`. DSP code may import the enums from `src/params.rs` and nothing else from NIH-plug. |
| B2 | **GUI: `nih_plug_egui`**, with every element painted by hand through `egui::Painter`. The concept document says vizia. This manual overrides it, because immediate-mode painting is simpler and every control needs custom drawing anyway. |
| B3 | **Smoothing is done inside the DSP structs** (`util::Smoother`), not with NIH-plug smoothers. `process()` reads each parameter once per host block with `.value()`. |
| B4 | **The chain runs in sub-blocks of 32 samples** (`util::SUB`). |
| B5 | **Oversampling**: Drive, Fuzz and Ring run together inside one oversampled section at the Quality factor (1×, 2× or 4×). |
| B6 | **Latency is a constant 24 samples at every Quality setting**. Lower factors are padded with delay. It is reported once in `initialize()` and never changes. |
| B7 | **Tuner**: YIN with a 2048-sample buffer, 1024-sample window, analysis every 1024 samples. Input above 48 kHz is decimated to about 48 kHz first. |
| B8 | **Note names use ASCII**: `C#`, and `b` / `#` for the flat and sharp signs. This avoids missing glyphs. |
| B9 | **MIDI**: Note On 60 = Loop, 61 = Stop, 62 = Clear (any channel). |
| B10 | **Momentary buttons** (Loop, Stop, Clear) are BoolParams. The audio thread acts on the rising edge. The GUI holds them true while the mouse is down. |
| B11 | **Licence: GPL-3.0-or-later**, because the VST3 bindings used by NIH-plug are GPLv3. The owner may change this before release (HUMAN CHECK). |
| B12 | **Editor size**: fixed at 1120 × 760 logical px. The host's DPI scaling is honoured. There is no scale menu in v1. |
| B13 | **Knob slots are a fixed 64 px wide**. Positions are computed by the layout helpers in Step S19. |
| B14 | **Fonts**: static instances generated from the Google variable fonts (Step S17). If that fails, the editor uses egui's default fonts and work continues. Fonts never block the project. |
| B15 | **Mono input**: in the 1-in / 2-out layout, channel 0 is copied into channel 1 at the top of `process()`. |
| B16 | **Plugin identity**: NAME `Holy Moly`, VENDOR `Holy Moly Audio`, VST3 class ID `*b"HolyMolyBoard001"`, subcategories Fx, Distortion, Delay, Modulation, Stereo. |

---

# PART C — REFERENCE DATA

## C1. Parameter table (52 parameters, IDs FROZEN)

Range types:
- **lin**: linear range.
- **log**: `FloatRange::Skewed` with the factor from `log_skew(min, max)` (Step S04).
- **pct**: stored as 0…max fraction, displayed as a percentage.

| # | ID | Rust field | Name | Kind | Range | Default | Display example |
|---|---|---|---|---|---|---|---|
| 1 | in_gain | `input` | Input | float lin dB | −24 … 24 | 0 | `0.0 dB`, `+6.0 dB` |
| 2 | out_gain | `output` | Output | float lin dB | −24 … 12 | 0 | `0.0 dB` |
| 3 | quality | `quality` | Quality | enum Quality | 1× / 2× / 4× | 4× | `4×` |
| 4 | tun_mute | `tuner.mute` | Mute | bool | | false | |
| 5 | tun_ref | `tuner.reference` | Reference | float lin | 430 … 450 | 440 | `440 Hz` |
| 6 | drv_on | `drive.on` | Drive On | bool | | true | |
| 7 | drv_dist | `drive.dist` | Dist | pct | 0 … 1 | 0.5 | `50%` |
| 8 | drv_tone | `drive.tone` | Tone | pct | 0 … 1 | 0.5 | `50%` |
| 9 | drv_level | `drive.level` | Level | float lin dB | −24 … 12 | 0 | `0.0 dB` |
| 10 | drv_voice | `drive.voice` | Voice | enum DriveVoice | Tight / Classic / Loose | Classic | `Classic` |
| 11 | fuz_on | `fuzz.on` | Fuzz On | bool | | true | |
| 12 | fuz_fuzz | `fuzz.fuzz` | Fuzz | pct | 0 … 1 | 0.6 | `60%` |
| 13 | fuz_bias | `fuzz.bias` | Bias | pct | 0 … 1 | 0.7 | `70%` |
| 14 | fuz_tone | `fuzz.tone` | Tone | pct | 0 … 1 | 0.5 | `50%` |
| 15 | fuz_volume | `fuzz.volume` | Volume | float lin dB | −24 … 12 | 0 | `0.0 dB` |
| 16 | rng_on | `ring.on` | Ring On | bool | | **false** | |
| 17 | rng_gain | `ring.gain` | Gain | float lin dB | 0 … 30 | 6 | `+6.0 dB` |
| 18 | rng_dist | `ring.dist` | Dist | pct | 0 … 1 | 0.4 | `40%` |
| 19 | rng_mod | `ring.modulation` | Mod | pct | 0 … 1 | 1.0 | `100%` |
| 20 | rng_output | `ring.output` | Output | float lin dB | −24 … 12 | 0 | `0.0 dB` |
| 21 | rng_mode | `ring.mode` | Mode | enum RingMode | Self / Osc | Self | `Self` |
| 22 | rng_freq | `ring.freq` | Freq | float log | 20 … 2000 | 440 | `440 Hz` |
| 23 | vib_on | `vibe.on` | Vibe On | bool | | true | |
| 24 | vib_rate | `vibe.rate` | Rate | float log | 0.1 … 10 | 1.5 | `1.5 Hz` |
| 25 | vib_depth | `vibe.depth` | Depth | pct | 0 … 1 | 0.6 | `60%` |
| 26 | vib_feedback | `vibe.feedback` | Feedback | pct | 0 … 0.9 | 0.3 | `30%` |
| 27 | vib_mix | `vibe.mix` | Mix | pct | 0 … 1 | 0.5 | `50%` |
| 28 | vib_mode | `vibe.mode` | Mode | enum VibeMode | Chorus / Vibrato | Chorus | `Chorus` |
| 29 | prz_on | `prism.on` | Prism On | bool | | **false** | |
| 30 | prz_pitch | `prism.pitch` | Pitch | float lin | −12 … 12 | 12 | `+12.0 st` |
| 31 | prz_primary | `prism.primary` | Primary | pct | 0 … 1 | 0.5 | `50%` |
| 32 | prz_secondary | `prism.secondary` | Secondary | pct | 0 … 1 | 0.0 | `0%` |
| 33 | prz_tracking | `prism.tracking` | Tracking | float lin ms | 10 … 120 | 40 | `40 ms` |
| 34 | prz_depth | `prism.depth` | Depth | pct | 0 … 1 | 0.2 | `20%` |
| 35 | prz_halo | `prism.halo` | Halo | pct | 0 … 0.95 | 0.0 | `0%` |
| 36 | prz_tone | `prism.tone` | Tone | float log | 1000 … 16000 | 8000 | `8.0 kHz` |
| 37 | ech_on | `echo.on` | Echo On | bool | | true | |
| 38 | ech_time | `echo.time` | Time | float log ms | 10 … 3000 | 400 | `400 ms`, `1.20 s` |
| 39 | ech_sync | `echo.sync` | Sync | bool | | false | |
| 40 | ech_division | `echo.division` | Division | enum Division | 13 values (C1a) | 1/4 | `1/4` |
| 41 | ech_feedback | `echo.feedback` | Feedback | pct | 0 … 1.1 | 0.35 | `35%` |
| 42 | ech_mix | `echo.mix` | Mix | pct | 0 … 1 | 0.35 | `35%` |
| 43 | ech_tone | `echo.tone` | Tone | float log | 1000 … 16000 | 6000 | `6.0 kHz` |
| 44 | ech_mod | `echo.modulation` | Mod | pct | 0 … 1 | 0.1 | `10%` |
| 45 | ech_mode | `echo.mode` | Mode | enum EchoMode | Digital / Tape / Multi / Reverse | Digital | `Digital` |
| 46 | ech_pingpong | `echo.pingpong` | Ping-pong | bool | | false | |
| 47 | ech_trails | `echo.trails` | Trails | bool | | true | |
| 48 | lop_loop | `looper.loop_btn` | Loop | bool (button) | | false | |
| 49 | lop_stop | `looper.stop_btn` | Stop | bool (button) | | false | |
| 50 | lop_clear | `looper.clear_btn` | Clear | bool (button) | | false | |
| 51 | lop_level | `looper.level` | Loop level | float lin dB | −60 … 6 | 0 | `0.0 dB`, `-inf dB` at −60 |
| 52 | lop_reverse | `looper.reverse` | Reverse | bool | | false | |

**C1a. Division values (in order) and their length in beats.** IDs are the lowercase names with `/` replaced by `_`.

`1/1`=4, `1/2D`=3, `1/2`=2, `1/2T`=4/3, `1/4D`=1.5, `1/4`=1, `1/4T`=2/3, `1/8D`=0.75, `1/8`=0.5, `1/8T`=1/3, `1/16D`=0.375, `1/16`=0.25, `1/16T`=1/6

**C1b. Display rules.** These must match `format` in the reference HTML.
- Percent: `round(v×100)` followed by `%`.
- dB: `round(v×10)/10` printed with one decimal and a `+` sign when > 0. Zero prints `0.0 dB`. Loop level ≤ −59.95 prints `-inf dB`.
- Hz: ≥ 1000 prints `x.x kHz`; < 10 prints `x.x Hz`; otherwise a rounded integer and ` Hz`.
- ms: ≥ 1000 prints `x.xx s`; otherwise a rounded integer and ` ms`.
- Semitones: like dB, with the unit ` st`.

## C2. Colour tokens

These are copied from the reference CSS. All hex values are sRGB.

| Token | Hex | Token | Hex |
|---|---|---|---|
| GROUND | #0a0c07 | ACID | #e4ff1a |
| DECK | #12150d | ACID_DIM | #5e6a0a |
| PANEL | #1b1f14 | ACID_DEEP | #252b06 |
| WELL | #050604 | ON_ACID | #0a0c07 |
| LINE | #2b3121 | DECAY | #ff6b1a |
| LINE_STRONG | #616d47 | DECAY_DEEP | #3a1606 |
| INK | #eef3d6 | ON_DECAY | #0a0c07 |
| INK_MUTED | #a8b18b | | |
| INK_FAINT | #687150 | | |

**Pedal finishes** (body colour / print colour):

| Finish | Body | Print |
|---|---|---|
| acid | #e4ff1a | #0a0c07 |
| black | #161912 | #e4ff1a |
| decay | #ff6b1a | #0a0c07 |
| bone | #e4e8cc | #1d5a2a |
| cherenkov | #0a2431 | #47d9ff |
| olive | #394020 | #e4ff1a |
| lead | #262b1e | #e4ff1a |

**Glow rule.** Wherever the reference uses a glow, draw the shape twice:
1. First, a version 3–4 px larger with the same colour at 25 % opacity (`color.gamma_multiply(0.25)`).
2. Then the normal shape on top.

**Disabled rule.** Multiply every colour of a disabled control by `gamma_multiply(0.32)`.

## C3. Fonts and text styles

Mono font: Martian Mono, condensed (width axis = 75). Display font: Big Shoulders Stencil.

| Style | Family name in code | Size px | Letter spacing (em) | Uppercase | Used for |
|---|---|---|---|---|---|
| WORDMARK | display900 | 40 | 0.02 | yes | "HOLY MOLY" |
| PEDAL_NAME | display800 | 26 | 0.03 | yes | pedal names |
| NOTE | display800 | 52 | 0 | yes | tuner note |
| LABEL | mono700 | 10 | 0.08 | yes | knob and selector labels, "MUTE" |
| VALUE | mono500 | 10 | 0.02 | no | knob value windows |
| OPTION | mono600 | 10 | 0.06 | yes | selector options, toggle and button text |
| READOUT | mono600 | 13 | 0.04 | yes | tuner cents, loop state, next-word |
| STAGE | mono500 | 10 | 0.10 | yes | stage badges, times, scale, sub-brand |

| Font file (in `assets/fonts/`) | Family name |
|---|---|
| `BigShouldersStencil-800.ttf` | display800 |
| `BigShouldersStencil-900.ttf` | display900 |
| `MartianMono-Condensed-500.ttf` | mono500 |
| `MartianMono-Condensed-600.ttf` | mono600 |
| `MartianMono-Condensed-700.ttf` | mono700 |

## C4. Editor geometry

All coordinates are logical px. The origin is the top-left of the 1120 × 760 editor. These numbers are authoritative; where they disagree with the HTML, use these.

### C4.1 Board

- Fill the whole editor with GROUND.
- Dot grid: 2 × 2 px squares in colour LINE, centred at (6 + 12i, 6 + 12j), for i = 0…93 and j = 0…63.
- Top strip rect: (16, 12, w 1088, h 112).
- Row 1 pedals: y = 136, h = 300. Row 2 pedals: y = 448, h = 300.

### C4.2 Pedals

| Pedal | x | y | w | Finish | Art | Stage | Default on |
|---|---|---|---|---|---|---|---|
| Drive | 16 | 136 | 240 | acid | stripes | 02 | yes |
| Fuzz | 268 | 136 | 232 | black | halftone | 03 | yes |
| Ring | 512 | 136 | 316 | decay | rings | 04 | no |
| Vibe | 840 | 136 | 264 | bone | diamonds | 05 | yes |
| Prism | 16 | 448 | 336 | cherenkov | fan | 06 | no |
| Echo | 364 | 448 | 476 | olive | echoes | 07 | yes |
| Looper | 852 | 448 | 252 | lead | orbit | 08 | custom LED |

**Pedal anatomy.** Let (x0, y0) be the pedal's top-left corner and w its width.

| Part | Geometry |
|---|---|
| Body | Rounded rect (x0, y0, w, 300), radius 6, fill = finish body. |
| Head row | Centre line y0+26. |
| Stage badge | Left at x0+12, top y0+19, height 14, width = text width + 8. 1 px outline in print colour, radius 2. Text in STAGE style, print colour, top y0+20. |
| Name | Left at badge right + 8, top y0+13. PEDAL_NAME style, print colour. |
| LED | Centre (x0+w−16, y0+26). |
| Body area | x from x0+12, width w−24; y from y0+40, height 184. |
| Foot area | y0+224 to y0+300. Footswitch centre (x0+w/2, y0+262). |
| Art area | Bottom 76 px of the pedal, drawn behind every control and clipped to the pedal body. |

**Rows inside each pedal's body area.** Rows are stacked and centred vertically with an 8 px gap. Inside a row, items are distributed with CSS `space-around` and an 8 px gap, and bottom-aligned (except Echo row 2, see below).

| Pedal | Row 1 | Row 2 |
|---|---|---|
| Drive | knobs drv_dist, drv_tone, drv_level | selector drv_voice |
| Fuzz | knobs fuz_fuzz, fuz_bias | knobs fuz_tone, fuz_volume |
| Ring | knobs rng_gain, rng_dist, rng_mod, rng_output | selector rng_mode, knob rng_freq (disabled when mode = Self) |
| Vibe | knobs vib_rate, vib_depth, vib_feedback | knob vib_mix (disabled when mode = Vibrato), selector vib_mode |
| Prism | knobs prz_pitch (bipolar), prz_primary, prz_secondary, prz_halo | knobs prz_tracking, prz_depth, prz_tone |
| Echo | knobs ech_time (disabled when sync), ech_division (13 steps, disabled when not sync), ech_feedback (danger zone above 100 %), ech_mix, ech_tone, ech_mod | `space-between`, vertically centred: selector ech_mode on the left; toggles SYNC, PING-PONG, TRAILS on the right, with 6 px gaps |
| Looper | loop display (full width 228, height 41) | knob lop_level, then a button column: REVERSE toggle on top, then STOP and CLEAR push buttons side by side with 6 px gaps; the toggle is stretched to the column width |

### C4.3 Item sizes

| Item | Width | Height | Notes |
|---|---|---|---|
| Knob sm | 64 | 78 | Dial box 46 (cap 28). |
| Knob md | 64 | 84 | Dial box 52 (cap 34). Used inside every pedal. |
| Knob lg | 64 | 92 | Dial box 60 (cap 42). |
| Knob column (all sizes) | | | Dial box at the top. Then 3 px gap, label (12 px high), 3 px gap, value window (14 px high). |
| Selector with label | 4 + Σ(option widths) + 2·(n−1) | 38 | Label 12, gap 4, box 22. |
| Selector option | 14 + text width | 18 | |
| Toggle | 26 + text width | 22 | |
| Push button | 32 + text width | 22 | |
| Footswitch | outer Ø 52, cap Ø 40 | | Small variant: outer Ø 34, cap Ø 26. |
| LED | Ø 8, plus a 2 px WELL ring | | |
| Loop display | 228 | 41 | |

### C4.4 Top strip

| Element | Geometry |
|---|---|
| Strip body | Rounded rect (16, 12, 1088, 112), radius 6, fill DECK, pedal shadow. |
| Hazard bar | y 120…124, x 16…1104. Diagonal stripes "/" in ACID and GROUND, each 6 px measured across the stripe (≈ 8.5 px horizontally). Clipped to the strip body. |
| Brand icon "halo" | 30 × 30 box at (32, 51), colour ACID with glow. |
| Wordmark "HOLY MOLY" | Top-left at (72, 41). WORDMARK style, ACID. |
| Sub-brand "STEREO PEDALBOARD · 8 STAGES" | Top-left at (72, 80). STAGE style, INK_MUTED. |
| Input knob (lg, in_gain, bipolar) | Column centre x 298, column top 20. Dial centre (298, 50). Label and value use INK_MUTED and ACID. |
| Tuner display | Rect (346, 24, 432, 84). Details in C4.5. |
| Ref knob (sm, tun_ref) | Column centre x 822, column top 27. Label text `REF`. |
| Mute column | Centre x 883. LED (decay tone) centre (883, 38). Small footswitch (decay tone, tun_mute) centre (883, 64). Text `MUTE` in LABEL style, INK_MUTED, centred at x 883, top 86. |
| Quality selector | Label `QUALITY` top-left at (916, 47). Selector box (916, 63, 92, 22). Options are 28 px wide at x = 918, 948, 978, y 65…83. |
| Output knob (lg, out_gain) | Column centre x 1056, column top 20. |

### C4.5 Tuner display (inside the rect (346, 24, 432, 84))

**Box.** Fill WELL, radius 4. When muted: add a 1 px DECAY outline, and a 4 px stripe bar along the top edge (DECAY and WELL diagonal stripes, 5 px each).

**Note column**, from x 358:
- Note name in NOTE style, top 42. Colour INK; ACID with glow when in tune. Show `–` when there is no signal.
- Octave number in STAGE style, INK_MUTED. Left = note right + 2; bottom aligned to y 84.

**Meter.** x 434…650, 21 segments, 2 px gaps.
- Segment width = (216 − 40) / 21 = 8.381. Segments span y 46…68, radius 2.
- Default fill: LINE. The centre segment (index 10) also gets a 1 px INK_FAINT outline.
- In tune (|cents| ≤ 5): segments 9, 10 and 11 are ACID with glow.
- Otherwise:
  - Let `at = round(cents/5)` and the lit index = 10 + at. That segment is DECAY with glow.
  - Segments strictly between the centre and the lit one are DECAY_DEEP.
- Scale labels in STAGE style, INK_MUTED, top 74: `b 50` left-aligned at x 434, `0` centred at x 542, `50 #` right-aligned at x 650.

**Side column.** Right-aligned at x 766.
- Cents line in READOUT style, top 50:
  - No signal: `NO SIGNAL` in INK_MUTED.
  - In tune: `IN TUNE` in ACID.
  - Otherwise: `+12 ct #` or `-12 ct b` in DECAY.
- Reference line `A4 = 440 Hz` in STAGE style, INK_MUTED, top 70.
- When muted: a `MUTED` badge (STAGE style text in ON_DECAY on a DECAY fill, padding 1 × 6, radius 2), top 86. In that case move the cents line to top 41 and the reference line to top 61.

---

# PART D — STEP INDEX

Copy this list into `PROGRESS.md` in Step S00.

```
[ ] S00 Workspace setup
[ ] S01 Cargo project and empty VST3 (SCAFFOLD)
[ ] S02 Pin the NIH-plug revision
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

# PART D2 — STEPS S00 TO S06

## D2.0 Addendum to Part A (binding)

**A7. File writing rule.** Create and edit files only with your file-editing tool. You SHALL NOT create source or config files with `>`, `Out-File`, `Set-Content` or `echo`. Windows PowerShell 5 writes UTF-16 or ANSI, and the Rust compiler rejects those files. The only exception is the `docs\API_NOTES.md` command in Step S02.

**A8. The CHECK command.** From Step S01 on, "CHECK" means running:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1
```

EXPECTED: the last line printed is `CHECK PASSED`.

**A9. Never run** `cargo update`, `cargo upgrade`, or `cargo add`. Never edit `Cargo.lock` by hand.

**A10. First builds are slow.** The first `cargo build` downloads and compiles about 300 crates. Wait up to 20 minutes. Do not cancel it, and do not "fix" a build that is still running.

---

### S00 — Workspace setup

**Goal:** a git repository with the reference files in place, plus PROGRESS.md and HUMAN_CHECKS.md.

**1. Check the tools.**

```powershell
git --version
rustc --version
cargo --version
```

EXPECTED: three version lines, with rustc 1.80 or newer.

If any command is missing:
1. Create `HUMAN_CHECKS.md` containing: `Install Git and Rust (https://rustup.rs, MSVC toolchain, accept the Visual Studio Build Tools prompt), then restart the agent.`
2. STOP.

**2. Create the folders and move the reference files.**

```powershell
New-Item -ItemType Directory -Force -Path reference, docs, src, src\dsp, src\editor, xtask\src, scripts, assets\fonts, .cargo | Out-Null
Move-Item -Path "holy-moly-editor.html" -Destination reference\
Move-Item -Path "Holy Moly*Concept.md" -Destination reference\
Get-ChildItem reference
```

EXPECTED: `reference` lists `holy-moly-editor.html` and the concept `.md` file. If a file was already in `reference\`, that is fine.

**3. Create `.gitignore`** (VERBATIM):

```
/target
*.log
```

**4. Create `PROGRESS.md`** (VERBATIM header; then paste the whole step list from Part D under "Steps"):

```markdown
# PROGRESS

Current step: S00

## Steps
(paste the list from plan.md Part D here)

## Notes
<!-- one line per finished step: "S0x: what was done, test count" -->

## Deviations
<!-- every minimal fix to a file from an earlier step: file, reason, change -->
```

**5. Create `HUMAN_CHECKS.md`** (VERBATIM):

```markdown
# HUMAN CHECKS — things only the owner can do

| # | From step | What to do | Result (owner fills in) |
|---|---|---|---|
```

**6. Initialise git.**

```powershell
git init
git config user.name "Holy Moly Agent"
git config user.email "agent@localhost"
git add -A
git commit -m "S00: workspace and references"
```

EXPECTED: the commit succeeds.

Tick S00 in PROGRESS.md and continue.

---

### S01 — Cargo project and empty VST3 (SCAFFOLD)

**Goal:** a plugin that builds, bundles and loads. It has no sound yet.

**Files to create** (all VERBATIM):

`Cargo.toml`
```toml
[package]
name = "holy_moly"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-or-later"
publish = false

[workspace]
members = ["xtask"]
resolver = "2"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", features = ["assert_process_allocs"] }
nih_plug_egui = { git = "https://github.com/robbert-vdh/nih-plug.git" }

[profile.release]
lto = "thin"
strip = "symbols"

[profile.profiling]
inherits = "release"
debug = true
strip = "none"
```

`.cargo/config.toml`
```toml
[alias]
xtask = "run --package xtask --release --"
```

`xtask/Cargo.toml`
```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git" }
```

`xtask/src/main.rs`
```rust
fn main() -> nih_plug_xtask::Result<()> {
    nih_plug_xtask::main()
}
```

`bundler.toml`
```toml
[holy_moly]
name = "Holy Moly"
```

`scripts/check.ps1`
```powershell
# Runs format, tests, clippy and the placeholder scan. Prints CHECK PASSED or CHECK FAILED.
cargo fmt
if ($LASTEXITCODE -ne 0) { Write-Host "CHECK FAILED: cargo fmt"; exit 1 }
cargo test --lib
if ($LASTEXITCODE -ne 0) { Write-Host "CHECK FAILED: cargo test"; exit 1 }
cargo clippy --lib -- -D warnings
if ($LASTEXITCODE -ne 0) { Write-Host "CHECK FAILED: cargo clippy"; exit 1 }
$hits = Get-ChildItem -Recurse -Path src -Filter *.rs | Select-String -Pattern 'todo!\(|unimplemented!\(|TODO|FIXME'
if ($hits) { $hits | ForEach-Object { Write-Host $_ }; Write-Host "CHECK FAILED: placeholders"; exit 1 }
Write-Host "CHECK PASSED"
```

`src/lib.rs`
```rust
// SCAFFOLD: replaced in Step S04 and again in Step S16.
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]

use nih_plug::prelude::*;
use std::sync::Arc;

pub struct HolyMoly {
    params: Arc<ScaffoldParams>,
}

#[derive(Params)]
struct ScaffoldParams {
    #[id = "scaffold_gain"]
    gain: FloatParam,
}

impl Default for HolyMoly {
    fn default() -> Self {
        Self {
            params: Arc::new(ScaffoldParams {
                gain: FloatParam::new("Scaffold", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 }),
            }),
        }
    }
}

impl Plugin for HolyMoly {
    const NAME: &'static str = "Holy Moly";
    const VENDOR: &'static str = "Holy Moly Audio";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for HolyMoly {
    const VST3_CLASS_ID: [u8; 16] = *b"HolyMolyBoard001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Delay,
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_vst3!(HolyMoly);
```

**VERIFY:**

```powershell
cargo build
```
EXPECTED: ends with `Finished`. This can take up to 20 minutes the first time (A10).

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1
```
EXPECTED: `CHECK PASSED` (it runs 0 tests at this point).

```powershell
cargo xtask bundle holy_moly --release
Test-Path "target\bundled\Holy Moly.vst3"
```
EXPECTED: `True`.

If the build fails, look the error up in Part E. Do not change the code in this step except as Part E instructs.

**Commit:** `S01: cargo project and scaffold plugin`

---

### S02 — Pin the NIH-plug revision and record the real APIs

**Goal:** the build always uses the same NIH-plug commit, and later steps can look up real function signatures in `docs\API_NOTES.md`.

**1. Read the hash that Cargo resolved.**

```powershell
$hash = (Select-String -Path Cargo.lock -Pattern 'nih-plug\.git#([0-9a-f]{40})' | Select-Object -First 1).Matches[0].Groups[1].Value
$hash
```

EXPECTED: one line of 40 hex characters.

**2. Edit `Cargo.toml`.** Add `rev = "<HASH>"` (the real hash) to both git dependencies. The result must look like this:

```toml
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "<HASH>", features = ["assert_process_allocs"] }
nih_plug_egui = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "<HASH>" }
```

**3. Edit `xtask/Cargo.toml`** the same way:

```toml
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "<HASH>" }
```

**4. Read the egui version.**

```powershell
(Select-String -Path Cargo.lock -Pattern '^name = "egui"$' -Context 0,1 | Select-Object -First 1).Context.PostContext
```

EXPECTED: one line such as `version = "0.31.1"`.

**5. Record the real APIs.** Run this whole block in one go:

```powershell
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
$root = Get-ChildItem (Join-Path $cargoHome "git\checkouts") -Directory -Filter "nih-plug-*" |
  ForEach-Object { Get-ChildItem $_.FullName -Directory } |
  Where-Object { $_.Name -eq $hash.Substring(0,7) } | Select-Object -First 1
$egui = Join-Path $root.FullName "nih_plug_egui\src\lib.rs"
$gui = Get-ChildItem -Recurse -Path $root.FullName -Filter *.rs | Select-String -Pattern 'pub struct ParamSetter' | Select-Object -First 1
$notes = @()
$notes += "# API notes (generated in S02)"
$notes += "NIH-plug source: $($root.FullName)"
$notes += "## create_egui_editor"
$notes += (Select-String -Path $egui -Pattern 'pub fn create_egui_editor' -Context 0,20 | Out-String)
$notes += "## EguiState::from_size"
$notes += (Select-String -Path $egui -Pattern 'pub fn from_size' -Context 0,4 | Out-String)
$notes += "## ParamSetter (file: $($gui.Path))"
$notes += (Select-String -Path $gui.Path -Pattern 'pub fn (begin_set_parameter|set_parameter|set_parameter_normalized|end_set_parameter)' -Context 0,6 | Out-String)
$notes | Set-Content -Encoding utf8 docs\API_NOTES.md
(Get-Content docs\API_NOTES.md).Count
```

EXPECTED: a number greater than 20. If `$root` is empty, write `checkout not found` into `docs\API_NOTES.md` by hand and continue.

**6. Create `docs/BUILD.md`:**

```markdown
# Build notes

- NIH-plug revision: <HASH>
- egui version (from Cargo.lock): <VERSION>
- Build: `cargo xtask bundle holy_moly --release` → `target\bundled\Holy Moly.vst3`
- Check: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1`
- Licence: GPL-3.0-or-later (plan.md B11)
```

**VERIFY:**

```powershell
cargo build
git diff --stat Cargo.lock
```

EXPECTED: `Finished`, and Cargo.lock is unchanged or shows only small changes. Then run CHECK → `CHECK PASSED`.

**Commit:** `S02: pin nih-plug revision, record APIs`

---

### S03 — `src/fmt.rs`: value text

**Goal:** one function per display rule in C1b. The host's automation lanes and the editor both use these functions.

**Files:**
- Create `src/fmt.rs` (VERBATIM, below).
- Edit `src/lib.rs`: add the line `pub mod fmt;` directly below the last `#![allow(...)]` line.

`src/fmt.rs`
```rust
//! Value text shared by the host (automation lanes) and the editor.
//! Matches `format` in reference/holy-moly-editor.html (plan.md C1b).

/// JavaScript `Math.round` (halves round up). Never returns -0.0.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor() + 0.0
}

pub fn percent(v: f32) -> String {
    let n = js_round(v as f64 * 100.0) as i64;
    format!("{n}%")
}

pub fn db(v: f32) -> String {
    with_unit(v, "dB")
}

pub fn semitones(v: f32) -> String {
    with_unit(v, "st")
}

fn with_unit(v: f32, unit: &str) -> String {
    let r = js_round(v as f64 * 10.0) / 10.0;
    let sign = if r > 0.0 { "+" } else { "" };
    format!("{sign}{r:.1} {unit}")
}

pub fn db_floor(v: f32, floor: f32) -> String {
    if v <= floor + 0.05 {
        "-inf dB".to_string()
    } else {
        db(v)
    }
}

/// Loop level: -60 dB and below prints as "-inf dB".
pub fn loop_level(v: f32) -> String {
    db_floor(v, -60.0)
}

pub fn hz(v: f32) -> String {
    if v >= 1000.0 {
        let k = v / 1000.0;
        format!("{k:.1} kHz")
    } else if v < 10.0 {
        format!("{v:.1} Hz")
    } else {
        let n = js_round(v as f64) as i64;
        format!("{n} Hz")
    }
}

pub fn ms(v: f32) -> String {
    if v >= 1000.0 {
        let s = v / 1000.0;
        format!("{s:.2} s")
    } else {
        let n = js_round(v as f64) as i64;
        format!("{n} ms")
    }
}

fn normalise(s: &str) -> String {
    s.trim().replace('\u{2212}', "-").to_lowercase()
}

/// The number at the start of `s`, e.g. "+6.0 dB" -> 6.0.
fn leading_number(s: &str) -> Option<f64> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        let ok = c.is_ascii_digit() || c == '.' || (i == 0 && (c == '-' || c == '+'));
        if !ok {
            break;
        }
        end = i + c.len_utf8();
    }
    s[..end].parse::<f64>().ok()
}

pub fn parse_percent(s: &str) -> Option<f32> {
    let s = normalise(s);
    leading_number(&s).map(|v| (v / 100.0) as f32)
}

pub fn parse_db(s: &str) -> Option<f32> {
    let s = normalise(s);
    leading_number(&s).map(|v| v as f32)
}

pub fn parse_semitones(s: &str) -> Option<f32> {
    parse_db(s)
}

pub fn parse_db_floor(s: &str, floor: f32) -> Option<f32> {
    let s = normalise(s);
    if s.contains("inf") {
        return Some(floor);
    }
    leading_number(&s).map(|v| v as f32)
}

pub fn parse_loop_level(s: &str) -> Option<f32> {
    parse_db_floor(s, -60.0)
}

pub fn parse_hz(s: &str) -> Option<f32> {
    let s = normalise(s);
    let v = leading_number(&s)?;
    let mult = if s.contains('k') { 1000.0 } else { 1.0 };
    Some((v * mult) as f32)
}

pub fn parse_ms(s: &str) -> Option<f32> {
    let s = normalise(s);
    let v = leading_number(&s)?;
    let mult = if s.contains("ms") || !s.ends_with('s') { 1.0 } else { 1000.0 };
    Some((v * mult) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Option<f32>, b: f32) -> bool {
        match a {
            Some(x) => (x - b).abs() <= 1e-4 * b.abs().max(1.0),
            None => false,
        }
    }

    #[test]
    fn percent_text() {
        assert_eq!(percent(0.5), "50%");
        assert_eq!(percent(0.0), "0%");
        assert_eq!(percent(1.1), "110%");
        assert_eq!(percent(0.35), "35%");
    }

    #[test]
    fn db_text() {
        assert_eq!(db(0.0), "0.0 dB");
        assert_eq!(db(6.0), "+6.0 dB");
        assert_eq!(db(-24.0), "-24.0 dB");
        assert_eq!(db(2.34), "+2.3 dB");
        assert_eq!(loop_level(-60.0), "-inf dB");
        assert_eq!(db_floor(-59.9, -60.0), "-59.9 dB");
    }

    #[test]
    fn hz_text() {
        assert_eq!(hz(440.0), "440 Hz");
        assert_eq!(hz(1500.0), "1.5 kHz");
        assert_eq!(hz(8000.0), "8.0 kHz");
        assert_eq!(hz(1.5), "1.5 Hz");
        assert_eq!(hz(20.0), "20 Hz");
    }

    #[test]
    fn ms_text() {
        assert_eq!(ms(400.0), "400 ms");
        assert_eq!(ms(1200.0), "1.20 s");
        assert_eq!(ms(10.0), "10 ms");
    }

    #[test]
    fn semitone_text() {
        assert_eq!(semitones(12.0), "+12.0 st");
        assert_eq!(semitones(-7.0), "-7.0 st");
        assert_eq!(semitones(0.0), "0.0 st");
    }

    #[test]
    fn parse_text() {
        assert!(close(parse_percent("50%"), 0.5));
        assert!(close(parse_percent(" 75 "), 0.75));
        assert!(parse_percent("abc").is_none());
        assert!(close(parse_db("+6.0 dB"), 6.0));
        assert!(close(parse_db("-3"), -3.0));
        assert!(close(parse_db("\u{2212}3 dB"), -3.0));
        assert!(close(parse_loop_level("-inf dB"), -60.0));
        assert!(close(parse_hz("1.5 kHz"), 1500.0));
        assert!(close(parse_hz("440 Hz"), 440.0));
        assert!(close(parse_ms("1.2 s"), 1200.0));
        assert!(close(parse_ms("400 ms"), 400.0));
        assert!(close(parse_ms("250"), 250.0));
        assert!(close(parse_semitones("+12.0 st"), 12.0));
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`. The test summary shows at least `6 passed` and `0 failed`.

**Commit:** `S03: value text formatters`

---

### S04 — `src/params.rs`: the 52 parameters

**Goal:** the complete parameter tree from Table C1, wired into the scaffold plugin so hosts see all 52 parameters.

**Files:**
- Create `src/params.rs` (VERBATIM, below).
- Replace `src/lib.rs` completely (VERBATIM, below).

`src/params.rs`
```rust
//! The 52 host parameters. IDs are FROZEN (plan.md Table C1). Smoothing is done in the DSP (B3).

use crate::fmt;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

pub const EDITOR_WIDTH: u32 = 1120;
pub const EDITOR_HEIGHT: u32 = 760;

/// Skew factor that puts the geometric mean of [min, max] at the knob centre.
pub fn log_skew(min: f32, max: f32) -> f32 {
    let mid = (min * max).sqrt();
    0.5f32.ln() / ((mid - min) / (max - min)).ln()
}

fn pct(name: &str, default: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max })
        .with_value_to_string(Arc::new(fmt::percent))
        .with_string_to_value(Arc::new(fmt::parse_percent))
}

fn db(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::db))
        .with_string_to_value(Arc::new(fmt::parse_db))
}

fn lin_hz(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::hz))
        .with_string_to_value(Arc::new(fmt::parse_hz))
}

fn log_hz(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Skewed { min, max, factor: log_skew(min, max) })
        .with_value_to_string(Arc::new(fmt::hz))
        .with_string_to_value(Arc::new(fmt::parse_hz))
}

fn lin_ms(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::ms))
        .with_string_to_value(Arc::new(fmt::parse_ms))
}

fn log_ms(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Skewed { min, max, factor: log_skew(min, max) })
        .with_value_to_string(Arc::new(fmt::ms))
        .with_string_to_value(Arc::new(fmt::parse_ms))
}

fn semis(name: &str, min: f32, max: f32, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(fmt::semitones))
        .with_string_to_value(Arc::new(fmt::parse_semitones))
}

fn loop_level(name: &str) -> FloatParam {
    FloatParam::new(name, 0.0, FloatRange::Linear { min: -60.0, max: 6.0 })
        .with_value_to_string(Arc::new(fmt::loop_level))
        .with_string_to_value(Arc::new(fmt::parse_loop_level))
}

// ---------------------------------------------------------------- enums

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    #[id = "1x"]
    #[name = "1×"]
    X1,
    #[id = "2x"]
    #[name = "2×"]
    X2,
    #[id = "4x"]
    #[name = "4×"]
    X4,
}

impl Quality {
    pub const ALL: [Quality; 3] = [Quality::X1, Quality::X2, Quality::X4];
    pub fn factor(self) -> usize {
        match self {
            Quality::X1 => 1,
            Quality::X2 => 2,
            Quality::X4 => 4,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Quality::X1 => "1×",
            Quality::X2 => "2×",
            Quality::X4 => "4×",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DriveVoice {
    #[id = "tight"]
    #[name = "Tight"]
    Tight,
    #[id = "classic"]
    #[name = "Classic"]
    Classic,
    #[id = "loose"]
    #[name = "Loose"]
    Loose,
}

impl DriveVoice {
    pub const ALL: [DriveVoice; 3] = [DriveVoice::Tight, DriveVoice::Classic, DriveVoice::Loose];
    pub fn label(self) -> &'static str {
        match self {
            DriveVoice::Tight => "Tight",
            DriveVoice::Classic => "Classic",
            DriveVoice::Loose => "Loose",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum RingMode {
    #[id = "self"]
    #[name = "Self"]
    SelfMod,
    #[id = "osc"]
    #[name = "Osc"]
    Osc,
}

impl RingMode {
    pub const ALL: [RingMode; 2] = [RingMode::SelfMod, RingMode::Osc];
    pub fn label(self) -> &'static str {
        match self {
            RingMode::SelfMod => "Self",
            RingMode::Osc => "Osc",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VibeMode {
    #[id = "chorus"]
    #[name = "Chorus"]
    Chorus,
    #[id = "vibrato"]
    #[name = "Vibrato"]
    Vibrato,
}

impl VibeMode {
    pub const ALL: [VibeMode; 2] = [VibeMode::Chorus, VibeMode::Vibrato];
    pub fn label(self) -> &'static str {
        match self {
            VibeMode::Chorus => "Chorus",
            VibeMode::Vibrato => "Vibrato",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum EchoMode {
    #[id = "digital"]
    #[name = "Digital"]
    Digital,
    #[id = "tape"]
    #[name = "Tape"]
    Tape,
    #[id = "multi"]
    #[name = "Multi"]
    Multi,
    #[id = "reverse"]
    #[name = "Reverse"]
    Reverse,
}

impl EchoMode {
    pub const ALL: [EchoMode; 4] = [EchoMode::Digital, EchoMode::Tape, EchoMode::Multi, EchoMode::Reverse];
    pub fn label(self) -> &'static str {
        match self {
            EchoMode::Digital => "Digital",
            EchoMode::Tape => "Tape",
            EchoMode::Multi => "Multi",
            EchoMode::Reverse => "Reverse",
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Division {
    #[id = "1_1"]
    #[name = "1/1"]
    Whole,
    #[id = "1_2d"]
    #[name = "1/2D"]
    HalfDotted,
    #[id = "1_2"]
    #[name = "1/2"]
    Half,
    #[id = "1_2t"]
    #[name = "1/2T"]
    HalfTriplet,
    #[id = "1_4d"]
    #[name = "1/4D"]
    QuarterDotted,
    #[id = "1_4"]
    #[name = "1/4"]
    Quarter,
    #[id = "1_4t"]
    #[name = "1/4T"]
    QuarterTriplet,
    #[id = "1_8d"]
    #[name = "1/8D"]
    EighthDotted,
    #[id = "1_8"]
    #[name = "1/8"]
    Eighth,
    #[id = "1_8t"]
    #[name = "1/8T"]
    EighthTriplet,
    #[id = "1_16d"]
    #[name = "1/16D"]
    SixteenthDotted,
    #[id = "1_16"]
    #[name = "1/16"]
    Sixteenth,
    #[id = "1_16t"]
    #[name = "1/16T"]
    SixteenthTriplet,
}

impl Division {
    pub const ALL: [Division; 13] = [
        Division::Whole,
        Division::HalfDotted,
        Division::Half,
        Division::HalfTriplet,
        Division::QuarterDotted,
        Division::Quarter,
        Division::QuarterTriplet,
        Division::EighthDotted,
        Division::Eighth,
        Division::EighthTriplet,
        Division::SixteenthDotted,
        Division::Sixteenth,
        Division::SixteenthTriplet,
    ];

    /// Length in quarter-note beats (plan.md C1a).
    pub fn beats(self) -> f32 {
        match self {
            Division::Whole => 4.0,
            Division::HalfDotted => 3.0,
            Division::Half => 2.0,
            Division::HalfTriplet => 4.0 / 3.0,
            Division::QuarterDotted => 1.5,
            Division::Quarter => 1.0,
            Division::QuarterTriplet => 2.0 / 3.0,
            Division::EighthDotted => 0.75,
            Division::Eighth => 0.5,
            Division::EighthTriplet => 1.0 / 3.0,
            Division::SixteenthDotted => 0.375,
            Division::Sixteenth => 0.25,
            Division::SixteenthTriplet => 1.0 / 6.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Division::Whole => "1/1",
            Division::HalfDotted => "1/2D",
            Division::Half => "1/2",
            Division::HalfTriplet => "1/2T",
            Division::QuarterDotted => "1/4D",
            Division::Quarter => "1/4",
            Division::QuarterTriplet => "1/4T",
            Division::EighthDotted => "1/8D",
            Division::Eighth => "1/8",
            Division::EighthTriplet => "1/8T",
            Division::SixteenthDotted => "1/16D",
            Division::Sixteenth => "1/16",
            Division::SixteenthTriplet => "1/16T",
        }
    }

    /// Position in `ALL` (0..=12).
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|d| *d == self).unwrap_or(5)
    }
}

// ---------------------------------------------------------------- parameter tree

#[derive(Params)]
pub struct HolyMolyParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "in_gain"]
    pub input: FloatParam,
    #[id = "out_gain"]
    pub output: FloatParam,
    #[id = "quality"]
    pub quality: EnumParam<Quality>,

    #[nested(group = "Tuner")]
    pub tuner: TunerParams,
    #[nested(group = "Drive")]
    pub drive: DriveParams,
    #[nested(group = "Fuzz")]
    pub fuzz: FuzzParams,
    #[nested(group = "Ring")]
    pub ring: RingParams,
    #[nested(group = "Vibe")]
    pub vibe: VibeParams,
    #[nested(group = "Prism")]
    pub prism: PrismParams,
    #[nested(group = "Echo")]
    pub echo: EchoParams,
    #[nested(group = "Looper")]
    pub looper: LooperParams,
}

impl Default for HolyMolyParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(EDITOR_WIDTH, EDITOR_HEIGHT),
            input: db("Input", -24.0, 24.0, 0.0),
            output: db("Output", -24.0, 12.0, 0.0),
            quality: EnumParam::new("Quality", Quality::X4),
            tuner: TunerParams::default(),
            drive: DriveParams::default(),
            fuzz: FuzzParams::default(),
            ring: RingParams::default(),
            vibe: VibeParams::default(),
            prism: PrismParams::default(),
            echo: EchoParams::default(),
            looper: LooperParams::default(),
        }
    }
}

#[derive(Params)]
pub struct TunerParams {
    #[id = "tun_mute"]
    pub mute: BoolParam,
    #[id = "tun_ref"]
    pub reference: FloatParam,
}

impl Default for TunerParams {
    fn default() -> Self {
        Self {
            mute: BoolParam::new("Mute", false),
            reference: lin_hz("Reference", 430.0, 450.0, 440.0),
        }
    }
}

#[derive(Params)]
pub struct DriveParams {
    #[id = "drv_on"]
    pub on: BoolParam,
    #[id = "drv_dist"]
    pub dist: FloatParam,
    #[id = "drv_tone"]
    pub tone: FloatParam,
    #[id = "drv_level"]
    pub level: FloatParam,
    #[id = "drv_voice"]
    pub voice: EnumParam<DriveVoice>,
}

impl Default for DriveParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Drive On", true),
            dist: pct("Dist", 0.5, 1.0),
            tone: pct("Tone", 0.5, 1.0),
            level: db("Level", -24.0, 12.0, 0.0),
            voice: EnumParam::new("Voice", DriveVoice::Classic),
        }
    }
}

#[derive(Params)]
pub struct FuzzParams {
    #[id = "fuz_on"]
    pub on: BoolParam,
    #[id = "fuz_fuzz"]
    pub fuzz: FloatParam,
    #[id = "fuz_bias"]
    pub bias: FloatParam,
    #[id = "fuz_tone"]
    pub tone: FloatParam,
    #[id = "fuz_volume"]
    pub volume: FloatParam,
}

impl Default for FuzzParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Fuzz On", true),
            fuzz: pct("Fuzz", 0.6, 1.0),
            bias: pct("Bias", 0.7, 1.0),
            tone: pct("Tone", 0.5, 1.0),
            volume: db("Volume", -24.0, 12.0, 0.0),
        }
    }
}

#[derive(Params)]
pub struct RingParams {
    #[id = "rng_on"]
    pub on: BoolParam,
    #[id = "rng_gain"]
    pub gain: FloatParam,
    #[id = "rng_dist"]
    pub dist: FloatParam,
    #[id = "rng_mod"]
    pub modulation: FloatParam,
    #[id = "rng_output"]
    pub output: FloatParam,
    #[id = "rng_mode"]
    pub mode: EnumParam<RingMode>,
    #[id = "rng_freq"]
    pub freq: FloatParam,
}

impl Default for RingParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Ring On", false),
            gain: db("Gain", 0.0, 30.0, 6.0),
            dist: pct("Dist", 0.4, 1.0),
            modulation: pct("Mod", 1.0, 1.0),
            output: db("Output", -24.0, 12.0, 0.0),
            mode: EnumParam::new("Mode", RingMode::SelfMod),
            freq: log_hz("Freq", 20.0, 2000.0, 440.0),
        }
    }
}

#[derive(Params)]
pub struct VibeParams {
    #[id = "vib_on"]
    pub on: BoolParam,
    #[id = "vib_rate"]
    pub rate: FloatParam,
    #[id = "vib_depth"]
    pub depth: FloatParam,
    #[id = "vib_feedback"]
    pub feedback: FloatParam,
    #[id = "vib_mix"]
    pub mix: FloatParam,
    #[id = "vib_mode"]
    pub mode: EnumParam<VibeMode>,
}

impl Default for VibeParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Vibe On", true),
            rate: log_hz("Rate", 0.1, 10.0, 1.5),
            depth: pct("Depth", 0.6, 1.0),
            feedback: pct("Feedback", 0.3, 0.9),
            mix: pct("Mix", 0.5, 1.0),
            mode: EnumParam::new("Mode", VibeMode::Chorus),
        }
    }
}

#[derive(Params)]
pub struct PrismParams {
    #[id = "prz_on"]
    pub on: BoolParam,
    #[id = "prz_pitch"]
    pub pitch: FloatParam,
    #[id = "prz_primary"]
    pub primary: FloatParam,
    #[id = "prz_secondary"]
    pub secondary: FloatParam,
    #[id = "prz_tracking"]
    pub tracking: FloatParam,
    #[id = "prz_depth"]
    pub depth: FloatParam,
    #[id = "prz_halo"]
    pub halo: FloatParam,
    #[id = "prz_tone"]
    pub tone: FloatParam,
}

impl Default for PrismParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Prism On", false),
            pitch: semis("Pitch", -12.0, 12.0, 12.0),
            primary: pct("Primary", 0.5, 1.0),
            secondary: pct("Secondary", 0.0, 1.0),
            tracking: lin_ms("Tracking", 10.0, 120.0, 40.0),
            depth: pct("Depth", 0.2, 1.0),
            halo: pct("Halo", 0.0, 0.95),
            tone: log_hz("Tone", 1000.0, 16000.0, 8000.0),
        }
    }
}

#[derive(Params)]
pub struct EchoParams {
    #[id = "ech_on"]
    pub on: BoolParam,
    #[id = "ech_time"]
    pub time: FloatParam,
    #[id = "ech_sync"]
    pub sync: BoolParam,
    #[id = "ech_division"]
    pub division: EnumParam<Division>,
    #[id = "ech_feedback"]
    pub feedback: FloatParam,
    #[id = "ech_mix"]
    pub mix: FloatParam,
    #[id = "ech_tone"]
    pub tone: FloatParam,
    #[id = "ech_mod"]
    pub modulation: FloatParam,
    #[id = "ech_mode"]
    pub mode: EnumParam<EchoMode>,
    #[id = "ech_pingpong"]
    pub pingpong: BoolParam,
    #[id = "ech_trails"]
    pub trails: BoolParam,
}

impl Default for EchoParams {
    fn default() -> Self {
        Self {
            on: BoolParam::new("Echo On", true),
            time: log_ms("Time", 10.0, 3000.0, 400.0),
            sync: BoolParam::new("Sync", false),
            division: EnumParam::new("Division", Division::Quarter),
            feedback: pct("Feedback", 0.35, 1.1),
            mix: pct("Mix", 0.35, 1.0),
            tone: log_hz("Tone", 1000.0, 16000.0, 6000.0),
            modulation: pct("Mod", 0.1, 1.0),
            mode: EnumParam::new("Mode", EchoMode::Digital),
            pingpong: BoolParam::new("Ping-pong", false),
            trails: BoolParam::new("Trails", true),
        }
    }
}

#[derive(Params)]
pub struct LooperParams {
    #[id = "lop_loop"]
    pub loop_btn: BoolParam,
    #[id = "lop_stop"]
    pub stop_btn: BoolParam,
    #[id = "lop_clear"]
    pub clear_btn: BoolParam,
    #[id = "lop_level"]
    pub level: FloatParam,
    #[id = "lop_reverse"]
    pub reverse: BoolParam,
}

impl Default for LooperParams {
    fn default() -> Self {
        Self {
            loop_btn: BoolParam::new("Loop", false),
            stop_btn: BoolParam::new("Stop", false),
            clear_btn: BoolParam::new("Clear", false),
            level: loop_level("Loop level"),
            reverse: BoolParam::new("Reverse", false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_IDS: [&str; 52] = [
        "in_gain", "out_gain", "quality", "tun_mute", "tun_ref", "drv_on", "drv_dist", "drv_tone",
        "drv_level", "drv_voice", "fuz_on", "fuz_fuzz", "fuz_bias", "fuz_tone", "fuz_volume",
        "rng_on", "rng_gain", "rng_dist", "rng_mod", "rng_output", "rng_mode", "rng_freq",
        "vib_on", "vib_rate", "vib_depth", "vib_feedback", "vib_mix", "vib_mode", "prz_on",
        "prz_pitch", "prz_primary", "prz_secondary", "prz_tracking", "prz_depth", "prz_halo",
        "prz_tone", "ech_on", "ech_time", "ech_sync", "ech_division", "ech_feedback", "ech_mix",
        "ech_tone", "ech_mod", "ech_mode", "ech_pingpong", "ech_trails", "lop_loop", "lop_stop",
        "lop_clear", "lop_level", "lop_reverse",
    ];

    #[test]
    fn has_52_params_with_frozen_ids() {
        let p = HolyMolyParams::default();
        let mut ids: Vec<String> = p.param_map().into_iter().map(|(id, _, _)| id).collect();
        ids.sort();
        let mut expected: Vec<String> = EXPECTED_IDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[test]
    fn defaults_match_table_c1() {
        let p = HolyMolyParams::default();
        assert_eq!(p.quality.value(), Quality::X4);
        assert!(!p.ring.on.value());
        assert!(!p.prism.on.value());
        assert!(p.drive.on.value());
        assert!(p.echo.trails.value());
        assert_eq!(p.echo.division.value(), Division::Quarter);
        assert_eq!(p.drive.voice.value(), DriveVoice::Classic);
        assert_eq!(p.ring.mode.value(), RingMode::SelfMod);
        assert!((p.echo.time.value() - 400.0).abs() < 1e-3);
        assert!((p.echo.feedback.value() - 0.35).abs() < 1e-6);
        assert!((p.prism.pitch.value() - 12.0).abs() < 1e-6);
        assert!((p.tuner.reference.value() - 440.0).abs() < 1e-6);
        assert!(p.looper.level.value().abs() < 1e-6);
    }

    #[test]
    fn log_skew_puts_geometric_mean_at_centre() {
        let (min, max) = (20.0f32, 2000.0f32);
        let f = log_skew(min, max);
        let norm = ((200.0 - min) / (max - min)).powf(f);
        assert!((norm - 0.5).abs() < 1e-4);
        assert_eq!(Division::Quarter.index(), 5);
        assert!((Division::EighthDotted.beats() - 0.75).abs() < 1e-6);
    }
}
```

`src/lib.rs` (replace the whole file)
```rust
// SCAFFOLD: process() is a pass-through until Step S16.
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]

pub mod fmt;
pub mod params;

use nih_plug::prelude::*;
use params::HolyMolyParams;
use std::sync::Arc;

pub struct HolyMoly {
    params: Arc<HolyMolyParams>,
}

impl Default for HolyMoly {
    fn default() -> Self {
        Self {
            params: Arc::new(HolyMolyParams::default()),
        }
    }
}

impl Plugin for HolyMoly {
    const NAME: &'static str = "Holy Moly";
    const VENDOR: &'static str = "Holy Moly Audio";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for HolyMoly {
    const VST3_CLASS_ID: [u8; 16] = *b"HolyMolyBoard001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Delay,
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_vst3!(HolyMoly);
```

**VERIFY:**
- CHECK → `CHECK PASSED`, with at least `9 passed`.
- BUNDLE → `Test-Path "target\bundled\Holy Moly.vst3"` prints `True`.

> **NOTE** — If the `Enum` derive rejects `#[name = ...]` or `#[id = ...]`, or `#[nested(group = ...)]` fails to compile, see Part E, entries E-P1 and E-P2. Do not rename IDs to make it compile.

**Commit:** `S04: 52 parameters with frozen IDs`

---

### S05 — `src/dsp/util.rs` and `src/dsp/testutil.rs`

**Goal:** the DSP building blocks that every later module uses, plus the helpers that tests use.

**Files:**
- Create `src/dsp/mod.rs`, `src/dsp/util.rs` and `src/dsp/testutil.rs` (VERBATIM, below).
- Edit `src/lib.rs`: add `pub mod dsp;` on the line directly below `pub mod params;`.

`src/dsp/mod.rs`
```rust
//! All DSP. Nothing in here imports nih_plug, except the enums from crate::params (plan.md B1).
pub mod util;

#[cfg(test)]
pub mod testutil;
```

`src/dsp/util.rs`
```rust
//! Shared DSP building blocks. Plain Rust, no nih-plug.

use std::f32::consts::PI;

/// The chain updates parameters every SUB samples (plan.md B4).
pub const SUB: usize = 32;

pub fn db_to_gain(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn gain_to_db(gain: f32) -> f32 {
    20.0 * gain.abs().max(1e-12).log10()
}

/// Blend from `dry` (amount 0) to `wet` (amount 1).
#[inline]
pub fn mix(dry: f32, wet: f32, amount: f32) -> f32 {
    dry + (wet - dry) * amount
}

/// Exponential smoother. `set_time(ms)` = time to cover about 99 % of a step.
#[derive(Clone, Copy, Debug)]
pub struct Smoother {
    current: f32,
    target: f32,
    coeff: f32,
}

impl Smoother {
    pub fn new(value: f32) -> Self {
        Self { current: value, target: value, coeff: 0.0 }
    }

    pub fn set_time(&mut self, sample_rate: f32, ms: f32) {
        let samples = ms * 0.001 * sample_rate;
        self.coeff = if samples < 1.0 { 0.0 } else { (-4.7 / samples).exp() };
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn reset(&mut self, value: f32) {
        self.current = value;
        self.target = value;
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    /// Moves `n` samples toward the target. Returns (value before, value after).
    pub fn block(&mut self, n: usize) -> (f32, f32) {
        let before = self.current;
        let k = self.coeff.powi(n as i32);
        self.current = self.target + (self.current - self.target) * k;
        if (self.current - self.target).abs() < 1e-6 {
            self.current = self.target;
        }
        (before, self.current)
    }
}

/// Linear interpolation across one sub-block: `set(from, to, ticks)`, then call `step()` `ticks` times.
#[derive(Clone, Copy, Debug, Default)]
pub struct Glide {
    value: f32,
    inc: f32,
}

impl Glide {
    pub fn set(&mut self, from: f32, to: f32, ticks: usize) {
        self.value = from;
        self.inc = (to - from) / ticks.max(1) as f32;
    }

    pub fn hold(&mut self, value: f32) {
        self.value = value;
        self.inc = 0.0;
    }

    #[inline]
    pub fn step(&mut self) -> f32 {
        self.value += self.inc;
        self.value
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// On/off with a 10 ms linear crossfade (footswitches, plan.md concept "Bypass").
#[derive(Clone, Copy, Debug)]
pub struct Switch {
    value: f32,
    target: f32,
    step: f32,
}

impl Switch {
    pub fn new(on: bool) -> Self {
        let v = if on { 1.0 } else { 0.0 };
        Self { value: v, target: v, step: 1.0 / 480.0 }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.step = 1.0 / (0.010 * sample_rate).max(1.0);
    }

    pub fn set(&mut self, on: bool) {
        self.target = if on { 1.0 } else { 0.0 };
    }

    pub fn reset(&mut self, on: bool) {
        self.set(on);
        self.value = self.target;
    }

    /// Moves `n` samples toward the target. Returns (value before, value after).
    pub fn block(&mut self, n: usize) -> (f32, f32) {
        let before = self.value;
        let delta = self.step * n as f32;
        self.value = if self.target > self.value {
            (self.value + delta).min(self.target)
        } else {
            (self.value - delta).max(self.target)
        };
        (before, self.value)
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_off(&self) -> bool {
        self.value == 0.0 && self.target == 0.0
    }

    pub fn is_on(&self) -> bool {
        self.value == 1.0 && self.target == 1.0
    }
}

/// First-order TPT filter. `tick` returns (lowpass, highpass); lowpass + highpass == input.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    g: f32,
    s: f32,
}

impl OnePole {
    pub fn set_cutoff(&mut self, sample_rate: f32, hz: f32) {
        let hz = hz.clamp(1.0, sample_rate * 0.49);
        let wc = (PI * hz / sample_rate).tan();
        self.g = wc / (1.0 + wc);
    }

    pub fn reset(&mut self) {
        self.s = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> (f32, f32) {
        let v = (x - self.s) * self.g;
        let lp = v + self.s;
        self.s = lp + v;
        (lp, x - lp)
    }

    #[inline]
    pub fn lowpass(&mut self, x: f32) -> f32 {
        self.tick(x).0
    }

    #[inline]
    pub fn highpass(&mut self, x: f32) -> f32 {
        self.tick(x).1
    }
}

/// RBJ biquad, transposed direct form II. Default = pass-through.
#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Self { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z1: 0.0, z2: 0.0 }
    }
}

impl Biquad {
    fn prep(sample_rate: f32, hz: f32, q: f32) -> (f32, f32) {
        let hz = hz.clamp(10.0, sample_rate * 0.45);
        let w0 = 2.0 * PI * hz / sample_rate;
        (w0, w0.sin() / (2.0 * q.max(0.1)))
    }

    fn set(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn set_lowpass(&mut self, sample_rate: f32, hz: f32, q: f32) {
        let (w0, alpha) = Self::prep(sample_rate, hz, q);
        let c = w0.cos();
        self.set((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, 1.0 + alpha, -2.0 * c, 1.0 - alpha);
    }

    pub fn set_highpass(&mut self, sample_rate: f32, hz: f32, q: f32) {
        let (w0, alpha) = Self::prep(sample_rate, hz, q);
        let c = w0.cos();
        self.set((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0, 1.0 + alpha, -2.0 * c, 1.0 - alpha);
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// DC blocker, corner at about 10 Hz.
#[derive(Clone, Copy, Debug)]
pub struct DcBlocker {
    r: f32,
    x1: f32,
    y1: f32,
}

impl Default for DcBlocker {
    fn default() -> Self {
        Self { r: 0.9987, x1: 0.0, y1: 0.0 }
    }
}

impl DcBlocker {
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.r = 1.0 - 2.0 * PI * 10.0 / sample_rate;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + self.r * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

/// Mono delay line with linear interpolation. Call `allocate` once outside the audio thread.
#[derive(Clone, Debug, Default)]
pub struct DelayLine {
    buf: Vec<f32>,
    pos: usize,
}

impl DelayLine {
    pub fn new() -> Self {
        Self { buf: Vec::new(), pos: 0 }
    }

    /// Allocates room for `max_samples` of delay. NOT for the audio thread.
    pub fn allocate(&mut self, max_samples: usize) {
        self.buf = vec![0.0; max_samples.max(4) + 4];
        self.pos = 0;
    }

    pub fn reset(&mut self) {
        self.buf.fill(0.0);
        self.pos = 0;
    }

    pub fn capacity(&self) -> usize {
        self.buf.len().saturating_sub(4)
    }

    #[inline]
    pub fn push(&mut self, x: f32) {
        if self.buf.is_empty() {
            return;
        }
        self.pos += 1;
        if self.pos == self.buf.len() {
            self.pos = 0;
        }
        self.buf[self.pos] = x;
    }

    /// Sample written `i` pushes ago (0 = newest).
    #[inline]
    pub fn tap(&self, i: usize) -> f32 {
        let n = self.buf.len();
        if n == 0 {
            return 0.0;
        }
        let idx = (self.pos + n - (i % n)) % n;
        self.buf[idx]
    }

    /// Fractional read, `delay` in samples (0 = newest). Clamped to [0, capacity].
    #[inline]
    pub fn read(&self, delay: f32) -> f32 {
        if self.buf.is_empty() {
            return 0.0;
        }
        let d = delay.clamp(0.0, self.capacity() as f32);
        let i = d.floor() as usize;
        let frac = d - i as f32;
        let a = self.tap(i);
        let b = self.tap(i + 1);
        a + (b - a) * frac
    }
}

/// Phase accumulator in [0, 1).
#[derive(Clone, Copy, Debug, Default)]
pub struct Phasor {
    phase: f32,
    inc: f32,
}

impl Phasor {
    pub fn set_freq(&mut self, sample_rate: f32, hz: f32) {
        self.inc = hz / sample_rate;
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase.rem_euclid(1.0);
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn phase(&self) -> f32 {
        self.phase
    }

    /// Returns the current phase, then advances by one sample.
    #[inline]
    pub fn step(&mut self) -> f32 {
        let p = self.phase;
        self.phase += self.inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        p
    }
}

/// xorshift32 noise.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    /// Uniform in [-1, 1].
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;

    const SR: f32 = 48000.0;

    fn filter_amp(f: &mut impl FnMut(f32) -> f32, freq: f32) -> f32 {
        let x = sine(freq, SR, 4800, 1.0);
        let y: Vec<f32> = x.iter().map(|&v| f(v)).collect();
        tone_amplitude(&y[480..], freq, SR)
    }

    #[test]
    fn db_conversions() {
        assert!((db_to_gain(6.0) - 1.9953).abs() < 1e-3);
        assert!((gain_to_db(0.5) + 6.0206).abs() < 1e-3);
        assert!((mix(1.0, 3.0, 0.5) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn smoother_reaches_99_percent_in_set_time() {
        let mut s = Smoother::new(0.0);
        s.set_time(SR, 20.0);
        s.set_target(1.0);
        for _ in 0..30 {
            s.block(SUB);
        }
        assert!(s.value() >= 0.99 && s.value() <= 1.0, "{}", s.value());
    }

    #[test]
    fn smoother_block_returns_before_and_after() {
        let mut s = Smoother::new(2.0);
        s.set_time(SR, 0.0);
        s.set_target(5.0);
        assert_eq!(s.block(32), (2.0, 5.0));
    }

    #[test]
    fn glide_steps_linearly() {
        let mut g = Glide::default();
        g.set(0.0, 1.0, 4);
        assert_eq!(g.step(), 0.25);
        assert_eq!(g.step(), 0.5);
        assert_eq!(g.step(), 0.75);
        assert_eq!(g.step(), 1.0);
    }

    #[test]
    fn switch_fades_off_within_512_samples() {
        let mut s = Switch::new(true);
        s.set_sample_rate(SR);
        s.set(false);
        let (before, after) = s.block(32);
        assert_eq!(before, 1.0);
        assert!(after < 1.0);
        for _ in 0..15 {
            s.block(32);
        }
        assert!(s.is_off());
    }

    #[test]
    fn one_pole_lowpass_is_minus_3db_at_cutoff_and_complementary() {
        let mut lp = OnePole::default();
        lp.set_cutoff(SR, 1000.0);
        let a = filter_amp(&mut |v| lp.lowpass(v), 1000.0);
        assert!((a - 0.7071).abs() < 0.02, "{a}");
        lp.reset();
        assert!(filter_amp(&mut |v| lp.lowpass(v), 100.0) > 0.99);
        lp.reset();
        let (l, h) = lp.tick(0.3);
        assert!((l + h - 0.3).abs() < 1e-6);
    }

    #[test]
    fn biquad_lowpass_and_highpass() {
        let mut f = Biquad::default();
        f.set_lowpass(SR, 1000.0, std::f32::consts::FRAC_1_SQRT_2);
        assert!((filter_amp(&mut |v| f.tick(v), 1000.0) - 0.7071).abs() < 0.02);
        f.reset();
        assert!(filter_amp(&mut |v| f.tick(v), 10000.0) < 0.02);
        f.reset();
        assert!(filter_amp(&mut |v| f.tick(v), 100.0) > 0.99);
        let mut h = Biquad::default();
        h.set_highpass(SR, 1000.0, std::f32::consts::FRAC_1_SQRT_2);
        assert!((filter_amp(&mut |v| h.tick(v), 1000.0) - 0.7071).abs() < 0.02);
    }

    #[test]
    fn delay_line_reads_back_with_interpolation() {
        let mut d = DelayLine::new();
        d.allocate(16);
        for i in 0..10 {
            d.push(i as f32);
        }
        assert_eq!(d.read(0.0), 9.0);
        assert_eq!(d.read(3.0), 6.0);
        assert!((d.read(2.5) - 6.5).abs() < 1e-6);
        d.reset();
        assert_eq!(d.read(1.0), 0.0);
        let empty = DelayLine::new();
        assert_eq!(empty.read(3.0), 0.0);
    }

    #[test]
    fn dc_blocker_removes_dc() {
        let mut b = DcBlocker::default();
        b.set_sample_rate(SR);
        let mut y = 1.0;
        for _ in 0..48000 {
            y = b.tick(1.0);
        }
        assert!(y.abs() < 0.01, "{y}");
    }

    #[test]
    fn rng_is_bounded_and_deterministic() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let mut sum_abs = 0.0;
        for _ in 0..1000 {
            let x = a.next_f32();
            assert!(x >= -1.0 && x <= 1.0);
            assert_eq!(x, b.next_f32());
            sum_abs += x.abs();
        }
        assert!(sum_abs / 1000.0 > 0.3);
    }

    #[test]
    fn phasor_wraps() {
        let mut p = Phasor::default();
        p.set_freq(SR, 12000.0);
        assert_eq!(p.step(), 0.0);
        assert_eq!(p.step(), 0.25);
        assert_eq!(p.step(), 0.5);
        assert_eq!(p.step(), 0.75);
        assert_eq!(p.step(), 0.0);
    }
}
```

`src/dsp/testutil.rs`
```rust
//! Helpers for unit tests only.

use std::f64::consts::PI;

pub fn sine(freq: f32, sr: f32, len: usize, amp: f32) -> Vec<f32> {
    (0..len)
        .map(|n| (amp as f64 * (2.0 * PI * freq as f64 * n as f64 / sr as f64).sin()) as f32)
        .collect()
}

pub fn noise(len: usize, amp: f32, seed: u32) -> Vec<f32> {
    let mut rng = crate::dsp::util::Rng::new(seed);
    (0..len).map(|_| rng.next_f32() * amp).collect()
}

/// Amplitude of the `freq` component (single-bin DFT). Use whole numbers of cycles for exact results.
pub fn tone_amplitude(x: &[f32], freq: f32, sr: f32) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    let w = 2.0 * PI * freq as f64 / sr as f64;
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for (n, &v) in x.iter().enumerate() {
        re += v as f64 * (w * n as f64).cos();
        im -= v as f64 * (w * n as f64).sin();
    }
    (2.0 * (re * re + im * im).sqrt() / x.len() as f64) as f32
}

pub fn rms(x: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    (x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64).sqrt() as f32
}

pub fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0f32, |m, v| m.max(v.abs()))
}

pub fn all_finite(x: &[f32]) -> bool {
    x.iter().all(|v| v.is_finite())
}

pub fn argmax_abs(x: &[f32]) -> usize {
    let mut best = 0;
    for i in 0..x.len() {
        if x[i].abs() > x[best].abs() {
            best = i;
        }
    }
    best
}

pub fn db(x: f32) -> f32 {
    20.0 * x.abs().max(1e-12).log10()
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `20 passed`.

**Commit:** `S05: dsp utilities and test helpers`

---

### S06 — `src/dsp/oversample.rs`

**Goal:** a stereo 1×/2×/4× oversampler with a constant latency of 24 samples (plan.md B5, B6).

**How it works** (read only; the code below does all of this):

| Item | Design |
|---|---|
| Stage A (rate ↔ 2× rate) | Linear-phase halfband FIR, 39 taps. Delay 19 base samples for up plus down. |
| Stage B (2× ↔ 4×) | Halfband FIR, 21 taps. Adds 5 base samples. |
| Padding | 4× needs 0 extra samples (19 + 5 = 24). 2× pads 5. 1× pads 24. |
| Changing factor | Clears all filter history. A short click is acceptable when Quality changes. |
| Callback | `f(left, right) -> (left, right)` is called once per oversampled stereo frame. |

**Files:**
- Create `src/dsp/oversample.rs` (VERBATIM, below).
- Edit `src/dsp/mod.rs`: add `pub mod oversample;` on the line directly below `pub mod util;`.

`src/dsp/oversample.rs`
```rust
//! Stereo 1x/2x/4x oversampler with constant 24-sample latency (plan.md B5, B6).

use std::f64::consts::PI;

/// Latency in base-rate samples at every factor.
pub const OS_LATENCY: usize = 24;

const TAPS_A: usize = 39;
const TAPS_B: usize = 21;

/// Windowed-sinc halfband lowpass (cutoff = quarter of the high rate), normalised so that
/// each polyphase branch has DC gain 0.5 exactly.
fn design_halfband(n: usize) -> Vec<f32> {
    let m = (n - 1) / 2;
    let mut h = vec![0.0f64; n];
    for i in 0..n {
        let k = i as f64 - m as f64;
        let x = PI * 0.5 * k;
        let sinc = if i == m { 1.0 } else { x.sin() / x };
        let t = i as f64 / (n - 1) as f64;
        let w = 0.42 - 0.5 * (2.0 * PI * t).cos() + 0.08 * (4.0 * PI * t).cos();
        h[i] = 0.5 * sinc * w;
    }
    let mut other_sum = 0.0;
    for i in 0..n {
        if (i as i64 - m as i64) % 2 != 0 {
            other_sum += h[i];
        }
    }
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let k = i as i64 - m as i64;
        out[i] = if k == 0 {
            0.5
        } else if k % 2 == 0 {
            0.0
        } else {
            (h[i] * 0.5 / other_sum) as f32
        };
    }
    out
}

struct Fir {
    taps: Vec<f32>,
    hist: Vec<f32>,
    pos: usize,
}

impl Fir {
    fn new(taps: Vec<f32>) -> Self {
        let n = taps.len();
        Self { taps, hist: vec![0.0; n], pos: 0 }
    }

    fn reset(&mut self) {
        self.hist.fill(0.0);
        self.pos = 0;
    }

    #[inline]
    fn push(&mut self, x: f32) {
        self.pos += 1;
        if self.pos == self.hist.len() {
            self.pos = 0;
        }
        self.hist[self.pos] = x;
    }

    #[inline]
    fn out(&self) -> f32 {
        let n = self.hist.len();
        let mut acc = 0.0f32;
        let mut idx = self.pos;
        for &t in &self.taps {
            acc += t * self.hist[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }
        acc
    }
}

/// One channel of one 2x stage.
struct Stage {
    up: Fir,
    down: Fir,
}

impl Stage {
    fn new(taps: usize) -> Self {
        let t = design_halfband(taps);
        Self { up: Fir::new(t.clone()), down: Fir::new(t) }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }

    #[inline]
    fn upsample(&mut self, x: f32) -> [f32; 2] {
        self.up.push(2.0 * x);
        let a = self.up.out();
        self.up.push(0.0);
        let b = self.up.out();
        [a, b]
    }

    /// Output is taken on the even phase so the delay stays an integer.
    #[inline]
    fn downsample(&mut self, a: f32, b: f32) -> f32 {
        self.down.push(a);
        let y = self.down.out();
        self.down.push(b);
        y
    }
}

struct Pad {
    buf: [f32; 32],
    pos: usize,
    delay: usize,
}

impl Pad {
    fn new() -> Self {
        Self { buf: [0.0; 32], pos: 0, delay: 0 }
    }

    fn reset(&mut self, delay: usize) {
        self.buf = [0.0; 32];
        self.pos = 0;
        self.delay = delay.min(31);
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        if self.delay == 0 {
            return x;
        }
        self.buf[self.pos] = x;
        let y = self.buf[(self.pos + 32 - self.delay) % 32];
        self.pos = (self.pos + 1) % 32;
        y
    }
}

struct OsChannel {
    a: Stage,
    b: Stage,
    pad: Pad,
}

impl OsChannel {
    fn new() -> Self {
        Self { a: Stage::new(TAPS_A), b: Stage::new(TAPS_B), pad: Pad::new() }
    }
}

pub struct Oversampler {
    ch: [OsChannel; 2],
    factor: usize,
}

impl Default for Oversampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Oversampler {
    /// Allocates. NOT for the audio thread.
    pub fn new() -> Self {
        let mut os = Self { ch: [OsChannel::new(), OsChannel::new()], factor: 4 };
        os.reset();
        os
    }

    pub fn factor(&self) -> usize {
        self.factor
    }

    pub fn reset(&mut self) {
        let delay = match self.factor {
            1 => 24,
            2 => 5,
            _ => 0,
        };
        for c in self.ch.iter_mut() {
            c.a.reset();
            c.b.reset();
            c.pad.reset(delay);
        }
    }

    /// Processes `left`/`right` in place. `f` runs once per oversampled stereo frame.
    pub fn process<F>(&mut self, factor: usize, left: &mut [f32], right: &mut [f32], mut f: F)
    where
        F: FnMut(f32, f32) -> (f32, f32),
    {
        let factor = match factor {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        if factor != self.factor {
            self.factor = factor;
            self.reset();
        }
        let n = left.len().min(right.len());
        let [cl, cr] = &mut self.ch;
        for i in 0..n {
            let (yl, yr) = if factor == 1 {
                f(left[i], right[i])
            } else if factor == 2 {
                let ul = cl.a.upsample(left[i]);
                let ur = cr.a.upsample(right[i]);
                let (l0, r0) = f(ul[0], ur[0]);
                let (l1, r1) = f(ul[1], ur[1]);
                (cl.a.downsample(l0, l1), cr.a.downsample(r0, r1))
            } else {
                let ul = cl.a.upsample(left[i]);
                let ur = cr.a.upsample(right[i]);
                let mut ml = [0.0f32; 2];
                let mut mr = [0.0f32; 2];
                for k in 0..2 {
                    let ql = cl.b.upsample(ul[k]);
                    let qr = cr.b.upsample(ur[k]);
                    let (l0, r0) = f(ql[0], qr[0]);
                    let (l1, r1) = f(ql[1], qr[1]);
                    ml[k] = cl.b.downsample(l0, l1);
                    mr[k] = cr.b.downsample(r0, r1);
                }
                (cl.a.downsample(ml[0], ml[1]), cr.a.downsample(mr[0], mr[1]))
            };
            left[i] = cl.pad.process(yl);
            right[i] = cr.pad.process(yr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;

    fn run(factor: usize, input: &[f32], mut f: impl FnMut(f32, f32) -> (f32, f32)) -> Vec<f32> {
        let mut os = Oversampler::new();
        let mut l = input.to_vec();
        let mut r = input.to_vec();
        for (cl, cr) in l.chunks_mut(32).zip(r.chunks_mut(32)) {
            os.process(factor, cl, cr, &mut f);
        }
        l
    }

    #[test]
    fn latency_is_24_at_every_factor() {
        for factor in [1usize, 2, 4] {
            let mut x = vec![0.0f32; 256];
            x[10] = 1.0;
            let y = run(factor, &x, |a, b| (a, b));
            assert_eq!(argmax_abs(&y), 10 + OS_LATENCY, "factor {factor}");
        }
    }

    #[test]
    fn dc_gain_is_unity() {
        for factor in [1usize, 2, 4] {
            let x = vec![0.5f32; 512];
            let y = run(factor, &x, |a, b| (a, b));
            assert!((y[511] - 0.5).abs() < 1e-3, "factor {factor}: {}", y[511]);
        }
    }

    #[test]
    fn passband_is_flat_at_1khz() {
        let sr = 48000.0;
        let x = sine(1000.0, sr, 4800, 0.5);
        for factor in [2usize, 4] {
            let y = run(factor, &x, |a, b| (a, b));
            let a = tone_amplitude(&y[480..4800], 1000.0, sr);
            assert!((db(a) - db(0.5)).abs() < 0.3, "factor {factor}: {a}");
        }
    }

    #[test]
    fn oversampling_reduces_aliasing() {
        let sr = 48000.0;
        let x = sine(15000.0, sr, 4800, 0.9);
        let cube = |a: f32, b: f32| (a * a * a, b * b * b);
        let y1 = run(1, &x, cube);
        let y4 = run(4, &x, cube);
        let alias1 = tone_amplitude(&y1[480..4800], 3000.0, sr);
        let alias4 = tone_amplitude(&y4[480..4800], 3000.0, sr);
        assert!(alias1 > 0.1, "sanity: alias at 1x should be strong, got {alias1}");
        assert!(db(alias4) < db(alias1) - 30.0, "1x {alias1}, 4x {alias4}");
    }

    #[test]
    fn reset_clears_history() {
        let mut os = Oversampler::new();
        let mut l = noise(256, 0.5, 1);
        let mut r = noise(256, 0.5, 2);
        os.process(4, &mut l, &mut r, |a, b| (a, b));
        os.reset();
        let mut zl = vec![0.0f32; 256];
        let mut zr = vec![0.0f32; 256];
        os.process(4, &mut zl, &mut zr, |a, b| (a, b));
        assert_eq!(peak(&zl), 0.0);
        assert_eq!(peak(&zr), 0.0);
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `25 passed`.

> **CAUTION** — If `oversampling_reduces_aliasing` or `latency_is_24_at_every_factor` fails, you did not copy the code VERBATIM. Compare your file with the block above, line by line. Do not change the test.

**Commit:** `S06: stereo oversampler with constant 24-sample latency`

# PART D3 — STEPS S07 TO S12 (EFFECT MODULES)

## D3.0 The module contract (read once; every module in S07–S12 follows it)

Every module file contains:
- one `XxxSettings` struct with plain values (copied from the parameters in Step S15);
- one `Xxx` struct with exactly the methods below.

| Method | Drive, Fuzz, Ring, Vibe | Prism, Echo | Rule |
|---|---|---|---|
| `new()` | yes | yes | May allocate small things. |
| `set_sample_rate(sr)` | yes | no | Never allocates. Safe to call from the audio thread. |
| `prepare(sr)` | no | yes | ALLOCATES delay lines. Only from `initialize()`. |
| `set(settings)` | yes | yes | Sets targets. Call once per sub-block, before `begin_block`. |
| `reset()` | yes | yes | Clears audio state and snaps every smoother to its target. |
| `is_off()` | yes | yes | True when fully bypassed; the chain may then skip the module. |
| `begin_block(frames)` | yes | yes | Advances smoothers by `frames`. |
| `process_frame(l, r) -> (l, r)` | yes | yes | Call EXACTLY `frames` times after each `begin_block(frames)`. |

> **NOTE** — Drive, Fuzz and Ring run INSIDE the oversampler. For them, `sr` is the oversampled rate and `frames` counts oversampled frames (base frames × Quality factor). Vibe, Prism and Echo run at the host rate.

> **NOTE** — When a module is off, `process_frame` returns its input bit-for-bit. Every module has a test that proves this.

> **CAUTION** — The numbers in these tests were derived for exactly the code given. If a test fails:
> 1. Compare your file with the VERBATIM block, line by line.
> 2. Re-copy it.
> 3. Never edit a test.

---

### S07 — `src/dsp/drive.rs`

**Design (read only):**

| Stage | What it does |
|---|---|
| Pre-emphasis | One-pole high-pass. Cutoff 600 Hz (Tight), 200 Hz (Classic), 40 Hz (Loose). |
| Gain | `dist` 0…1 maps to 0…+50 dB. |
| Clipper | Tight: `clamp(±1)`. Classic: `tanh(x+0.2) − tanh(0.2)` (asymmetric). Loose: `x/√(1+x²)`. Output × 0.5. |
| DC blocker | 10 Hz. Removes the offset from the asymmetric clip. |
| Tilt tone | Split at 800 Hz. `t = 2·tone − 1`; low band × `−9t` dB, high band × `+9t` dB. |
| Level | dB gain. |

**Files:**
- Create `src/dsp/drive.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod drive;` below `pub mod oversample;`.

`src/dsp/drive.rs`
```rust
//! Drive (after the Suhr Riot): pre-emphasis high-pass, up to +50 dB of gain, a clipper,
//! a tilt tone control around 800 Hz and an output level.
//! Runs inside the oversampler: `set_sample_rate` receives the OVERSAMPLED rate.

use crate::dsp::util::{db_to_gain, mix, DcBlocker, Glide, OnePole, Smoother, Switch};
use crate::params::DriveVoice;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DriveSettings {
    pub on: bool,
    pub dist: f32,
    pub tone: f32,
    pub level_db: f32,
    pub voice: DriveVoice,
}

impl Default for DriveSettings {
    fn default() -> Self {
        Self { on: true, dist: 0.5, tone: 0.5, level_db: 0.0, voice: DriveVoice::Classic }
    }
}

/// Bass cut before the clipper, per voice.
fn hpf_cutoff(voice: DriveVoice) -> f32 {
    match voice {
        DriveVoice::Tight => 600.0,
        DriveVoice::Classic => 200.0,
        DriveVoice::Loose => 40.0,
    }
}

/// Tight: hard symmetric. Classic: asymmetric soft. Loose: softest.
#[inline]
fn clip(voice: DriveVoice, x: f32) -> f32 {
    match voice {
        DriveVoice::Tight => x.clamp(-1.0, 1.0),
        DriveVoice::Classic => (x + 0.2).tanh() - 0.2f32.tanh(),
        DriveVoice::Loose => x / (1.0 + x * x).sqrt(),
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    hpf: OnePole,
    dc: DcBlocker,
    tilt: OnePole,
}

impl Channel {
    fn clear(&mut self) {
        self.hpf.reset();
        self.dc.reset();
        self.tilt.reset();
    }

    #[inline]
    fn tick(&mut self, x: f32, gain: f32, voice: DriveVoice, lo: f32, hi: f32) -> f32 {
        let u = self.hpf.highpass(x);
        let c = self.dc.tick(clip(voice, gain * u) * 0.5);
        let (lp, hp) = self.tilt.tick(c);
        lp * lo + hp * hi
    }
}

pub struct Drive {
    sr: f32,
    s: DriveSettings,
    switch: Switch,
    dist: Smoother,
    tone: Smoother,
    level: Smoother,
    mix_g: Glide,
    gain_g: Glide,
    level_g: Glide,
    lo: f32,
    hi: f32,
    ch: [Channel; 2],
}

impl Drive {
    pub fn new() -> Self {
        let s = DriveSettings::default();
        let mut d = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            dist: Smoother::new(s.dist),
            tone: Smoother::new(s.tone),
            level: Smoother::new(s.level_db),
            mix_g: Glide::default(),
            gain_g: Glide::default(),
            level_g: Glide::default(),
            lo: 1.0,
            hi: 1.0,
            ch: [Channel::default(); 2],
        };
        d.set_sample_rate(48000.0);
        d.reset();
        d
    }

    /// `sr` is the OVERSAMPLED rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [&mut self.dist, &mut self.tone, &mut self.level] {
            sm.set_time(sr, 20.0);
        }
        let fc = hpf_cutoff(self.s.voice);
        for c in self.ch.iter_mut() {
            c.hpf.set_cutoff(sr, fc);
            c.dc.set_sample_rate(sr);
            c.tilt.set_cutoff(sr, 800.0);
        }
    }

    pub fn set(&mut self, s: DriveSettings) {
        let voice_changed = s.voice != self.s.voice;
        self.s = s;
        self.switch.set(s.on);
        self.dist.set_target(s.dist);
        self.tone.set_target(s.tone);
        self.level.set_target(s.level_db);
        if voice_changed {
            let (sr, fc) = (self.sr, hpf_cutoff(s.voice));
            for c in self.ch.iter_mut() {
                c.hpf.set_cutoff(sr, fc);
            }
        }
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.dist.reset(self.s.dist);
        self.tone.reset(self.s.tone);
        self.level.reset(self.s.level_db);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.update_tilt();
        self.mix_g.hold(self.switch.value());
        self.gain_g.hold(db_to_gain(50.0 * self.dist.value()));
        self.level_g.hold(db_to_gain(self.level.value()));
    }

    fn update_tilt(&mut self) {
        let t = self.tone.value() * 2.0 - 1.0;
        self.lo = db_to_gain(-9.0 * t);
        self.hi = db_to_gain(9.0 * t);
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        let (d0, d1) = self.dist.block(frames);
        self.gain_g.set(db_to_gain(50.0 * d0), db_to_gain(50.0 * d1), frames);
        let (l0, l1) = self.level.block(frames);
        self.level_g.set(db_to_gain(l0), db_to_gain(l1), frames);
        self.tone.block(frames);
        self.update_tilt();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let g = self.gain_g.step();
        let lv = self.level_g.step();
        if m == 0.0 {
            return (l, r);
        }
        let v = self.s.voice;
        let yl = self.ch[0].tick(l, g, v, self.lo, self.hi) * lv;
        let yr = self.ch[1].tick(r, g, v, self.lo, self.hi) * lv;
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: DriveSettings) -> Drive {
        let mut d = Drive::new();
        d.set_sample_rate(SR);
        d.set(s);
        d.reset();
        d
    }

    fn run(d: &mut Drive, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            d.begin_block(chunk.len());
            for &v in chunk {
                out.push(d.process_frame(v, v).0);
            }
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut d = make(DriveSettings { on: false, ..Default::default() });
        let x = noise(1000, 0.5, 3);
        assert_eq!(run(&mut d, &x), x);
    }

    fn h3_ratio(dist: f32) -> f32 {
        let mut d = make(DriveSettings { dist, ..Default::default() });
        let y = run(&mut d, &sine(200.0, SR, 52800, 0.1));
        let w = &y[4800..52800];
        tone_amplitude(w, 600.0, SR) / tone_amplitude(w, 200.0, SR)
    }

    #[test]
    fn more_dist_means_more_harmonics() {
        let low = h3_ratio(0.0);
        let high = h3_ratio(1.0);
        assert!(low < 0.05, "dist 0: {low}");
        assert!(high > 0.2, "dist 1: {high}");
    }

    #[test]
    fn level_is_applied_in_db() {
        let x = sine(200.0, SR, 9600, 0.1);
        let mut a = make(DriveSettings { level_db: 0.0, ..Default::default() });
        let mut b = make(DriveSettings { level_db: 12.0, ..Default::default() });
        let ya = run(&mut a, &x);
        let yb = run(&mut b, &x);
        let ratio = rms(&yb[4800..]) / rms(&ya[4800..]);
        assert!((ratio - db_to_gain(12.0)).abs() < 0.05, "{ratio}");
    }

    #[test]
    fn tight_cuts_more_bass_than_loose() {
        let x = sine(80.0, SR, 52800, 0.001);
        let amp = |voice: DriveVoice| {
            let mut d = make(DriveSettings { dist: 0.0, voice, ..Default::default() });
            let y = run(&mut d, &x);
            tone_amplitude(&y[4800..52800], 80.0, SR)
        };
        let tight = amp(DriveVoice::Tight);
        let loose = amp(DriveVoice::Loose);
        assert!(tight < loose * 0.5, "tight {tight}, loose {loose}");
    }

    #[test]
    fn tone_tilts_treble() {
        let x = sine(5000.0, SR, 52800, 0.01);
        let amp = |tone: f32| {
            let mut d = make(DriveSettings { dist: 0.0, tone, ..Default::default() });
            let y = run(&mut d, &x);
            tone_amplitude(&y[4800..52800], 5000.0, SR)
        };
        assert!(db(amp(1.0)) > db(amp(0.0)) + 10.0);
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 1.0, 9);
        for voice in DriveVoice::ALL {
            for (dist, tone, level) in [(0.0, 0.0, -24.0), (1.0, 1.0, 12.0), (1.0, 0.0, 12.0)] {
                let mut d = make(DriveSettings { on: true, dist, tone, level_db: level, voice });
                let y = run(&mut d, &x);
                assert!(all_finite(&y));
                assert!(peak(&y) < 20.0, "{voice:?} {dist} {tone} {level}: {}", peak(&y));
            }
        }
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `31 passed`.

**Commit:** `S07: drive module`

---

### S08 — `src/dsp/fuzz.rs`

**Design (read only):**

| Stage | What it does |
|---|---|
| Coupling | 80 Hz one-pole high-pass. |
| Gain | `fuzz` 0…1 maps to 0…+40 dB. |
| Bias gate | `gate = (1−bias)²·0.5`. Signal below the gate is removed (sputter). |
| Shaper | Positive side `tanh(v)`; negative side `0.7·tanh(v/0.7)`. Offset `0.25·(1−bias)`. |
| After the shaper | DC blocker, then a 2-pole low-pass at `1000·10^tone` Hz (1–10 kHz), output × 0.5, then volume. |

**Files:**
- Create `src/dsp/fuzz.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod fuzz;` below `pub mod drive;`.

`src/dsp/fuzz.rs`
```rust
//! Fuzz: coupling high-pass, gain, a bias-controlled gate and an asymmetric transistor-style
//! shaper, DC blocker, low-pass tone (1–10 kHz) and volume.
//! Runs inside the oversampler: `set_sample_rate` receives the OVERSAMPLED rate.

use crate::dsp::util::{db_to_gain, mix, Biquad, DcBlocker, Glide, OnePole, Smoother, Switch};
use std::f32::consts::FRAC_1_SQRT_2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuzzSettings {
    pub on: bool,
    pub fuzz: f32,
    pub bias: f32,
    pub tone: f32,
    pub volume_db: f32,
}

impl Default for FuzzSettings {
    fn default() -> Self {
        Self { on: true, fuzz: 0.6, bias: 0.7, tone: 0.5, volume_db: 0.0 }
    }
}

#[inline]
fn shape(v: f32) -> f32 {
    if v >= 0.0 {
        v.tanh()
    } else {
        0.7 * (v / 0.7).tanh()
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    hpf: OnePole,
    dc: DcBlocker,
    lp: Biquad,
}

impl Channel {
    fn clear(&mut self) {
        self.hpf.reset();
        self.dc.reset();
        self.lp.reset();
    }

    #[inline]
    fn tick(&mut self, x: f32, gain: f32, gate: f32, offset: f32) -> f32 {
        let pre = gain * self.hpf.highpass(x);
        let s = pre.signum() * (pre.abs() - gate).max(0.0);
        let y = shape(s + offset) - shape(offset);
        self.lp.tick(self.dc.tick(y)) * 0.5
    }
}

pub struct Fuzz {
    sr: f32,
    s: FuzzSettings,
    switch: Switch,
    fuzz: Smoother,
    bias: Smoother,
    tone: Smoother,
    volume: Smoother,
    mix_g: Glide,
    gain_g: Glide,
    vol_g: Glide,
    gate: f32,
    offset: f32,
    ch: [Channel; 2],
}

impl Fuzz {
    pub fn new() -> Self {
        let s = FuzzSettings::default();
        let mut f = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            fuzz: Smoother::new(s.fuzz),
            bias: Smoother::new(s.bias),
            tone: Smoother::new(s.tone),
            volume: Smoother::new(s.volume_db),
            mix_g: Glide::default(),
            gain_g: Glide::default(),
            vol_g: Glide::default(),
            gate: 0.0,
            offset: 0.0,
            ch: [Channel::default(); 2],
        };
        f.set_sample_rate(48000.0);
        f.reset();
        f
    }

    /// `sr` is the OVERSAMPLED rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [&mut self.fuzz, &mut self.bias, &mut self.tone, &mut self.volume] {
            sm.set_time(sr, 20.0);
        }
        for c in self.ch.iter_mut() {
            c.hpf.set_cutoff(sr, 80.0);
            c.dc.set_sample_rate(sr);
        }
        self.apply_block_values();
    }

    pub fn set(&mut self, s: FuzzSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.fuzz.set_target(s.fuzz);
        self.bias.set_target(s.bias);
        self.tone.set_target(s.tone);
        self.volume.set_target(s.volume_db);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.fuzz.reset(self.s.fuzz);
        self.bias.reset(self.s.bias);
        self.tone.reset(self.s.tone);
        self.volume.reset(self.s.volume_db);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
        self.gain_g.hold(db_to_gain(40.0 * self.fuzz.value()));
        self.vol_g.hold(db_to_gain(self.volume.value()));
    }

    fn apply_block_values(&mut self) {
        let b = self.bias.value();
        self.gate = (1.0 - b) * (1.0 - b) * 0.5;
        self.offset = 0.25 * (1.0 - b);
        let fc = 1000.0 * 10.0f32.powf(self.tone.value());
        let sr = self.sr;
        for c in self.ch.iter_mut() {
            c.lp.set_lowpass(sr, fc, FRAC_1_SQRT_2);
        }
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        let (f0, f1) = self.fuzz.block(frames);
        self.gain_g.set(db_to_gain(40.0 * f0), db_to_gain(40.0 * f1), frames);
        let (v0, v1) = self.volume.block(frames);
        self.vol_g.set(db_to_gain(v0), db_to_gain(v1), frames);
        self.bias.block(frames);
        self.tone.block(frames);
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let g = self.gain_g.step();
        let v = self.vol_g.step();
        if m == 0.0 {
            return (l, r);
        }
        let yl = self.ch[0].tick(l, g, self.gate, self.offset) * v;
        let yr = self.ch[1].tick(r, g, self.gate, self.offset) * v;
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: FuzzSettings) -> Fuzz {
        let mut f = Fuzz::new();
        f.set_sample_rate(SR);
        f.set(s);
        f.reset();
        f
    }

    fn run(f: &mut Fuzz, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            f.begin_block(chunk.len());
            for &v in chunk {
                out.push(f.process_frame(v, v).0);
            }
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut f = make(FuzzSettings { on: false, ..Default::default() });
        let x = noise(1000, 0.5, 13);
        assert_eq!(run(&mut f, &x), x);
    }

    #[test]
    fn low_bias_gates_small_signals() {
        let x = sine(200.0, SR, 9600, 0.005);
        let level = |bias: f32| {
            let mut f = make(FuzzSettings { fuzz: 0.0, bias, tone: 1.0, ..Default::default() });
            let y = run(&mut f, &x);
            rms(&y[4800..])
        };
        let starved = level(0.0);
        let full = level(1.0);
        assert!(full > 0.0005, "{full}");
        assert!(starved < full * 0.1, "starved {starved}, full {full}");
    }

    #[test]
    fn tone_controls_treble() {
        let x = sine(6000.0, SR, 52800, 0.01);
        let amp = |tone: f32| {
            let mut f = make(FuzzSettings { fuzz: 0.0, bias: 1.0, tone, ..Default::default() });
            let y = run(&mut f, &x);
            tone_amplitude(&y[4800..52800], 6000.0, SR)
        };
        assert!(db(amp(1.0)) > db(amp(0.0)) + 15.0);
    }

    #[test]
    fn more_fuzz_means_more_harmonics() {
        let ratio = |fuzz: f32| {
            let mut f = make(FuzzSettings { fuzz, bias: 1.0, tone: 1.0, ..Default::default() });
            let y = run(&mut f, &sine(200.0, SR, 52800, 0.1));
            let w = &y[4800..52800];
            tone_amplitude(w, 600.0, SR) / tone_amplitude(w, 200.0, SR)
        };
        let low = ratio(0.0);
        let high = ratio(1.0);
        assert!(low < 0.05, "{low}");
        assert!(high > 0.15, "{high}");
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 1.0, 14);
        for (fuzz, bias, tone, vol) in [(0.0, 0.0, 0.0, -24.0), (1.0, 1.0, 1.0, 12.0), (1.0, 0.0, 0.5, 12.0)] {
            let mut f = make(FuzzSettings { on: true, fuzz, bias, tone, volume_db: vol });
            let y = run(&mut f, &x);
            assert!(all_finite(&y));
            assert!(peak(&y) < 10.0, "{}", peak(&y));
        }
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `36 passed`.

**Commit:** `S08: fuzz module`

---

### S09 — `src/dsp/ring.rs`

**Design (read only):**

| Stage | What it does |
|---|---|
| Gain | `a = gain_dB · x`. |
| Carrier | Osc: `sin(2π·freq·t)`. Self: `tanh(3a)` (a clipped copy of the signal). |
| Mod blend | `w = mix(tanh(a), a·carrier, mod)`. |
| Dist | `d = 1 + 9·dist`; `y = tanh(d·w)/√d`. |
| Output | DC blocker (Self mode creates DC), then output dB. |

**Files:**
- Create `src/dsp/ring.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod ring;` below `pub mod fuzz;`.

`src/dsp/ring.rs`
```rust
//! Ring (after the DOD Gonkulator): Self = signal x clipped copy of itself, Osc = signal x sine.
//! Runs inside the oversampler: `set_sample_rate` receives the OVERSAMPLED rate.

use crate::dsp::util::{db_to_gain, mix, DcBlocker, Glide, Phasor, Smoother, Switch};
use crate::params::RingMode;
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingSettings {
    pub on: bool,
    pub gain_db: f32,
    pub dist: f32,
    pub modulation: f32,
    pub output_db: f32,
    pub mode: RingMode,
    pub freq_hz: f32,
}

impl Default for RingSettings {
    fn default() -> Self {
        Self {
            on: false,
            gain_db: 6.0,
            dist: 0.4,
            modulation: 1.0,
            output_db: 0.0,
            mode: RingMode::SelfMod,
            freq_hz: 440.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Channel {
    dc: DcBlocker,
}

impl Channel {
    #[inline]
    fn tick(&mut self, x: f32, gain: f32, osc: f32, mode: RingMode, modulation: f32, d: f32) -> f32 {
        let a = gain * x;
        let carrier = match mode {
            RingMode::Osc => osc,
            RingMode::SelfMod => (3.0 * a).tanh(),
        };
        let w = mix(a.tanh(), a * carrier, modulation);
        self.dc.tick((d * w).tanh() / d.sqrt())
    }
}

pub struct Ring {
    sr: f32,
    s: RingSettings,
    switch: Switch,
    gain: Smoother,
    dist: Smoother,
    modulation: Smoother,
    output: Smoother,
    freq: Smoother,
    mix_g: Glide,
    gain_g: Glide,
    out_g: Glide,
    d: f32,
    modu: f32,
    osc: Phasor,
    ch: [Channel; 2],
}

impl Ring {
    pub fn new() -> Self {
        let s = RingSettings::default();
        let mut r = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            gain: Smoother::new(s.gain_db),
            dist: Smoother::new(s.dist),
            modulation: Smoother::new(s.modulation),
            output: Smoother::new(s.output_db),
            freq: Smoother::new(s.freq_hz),
            mix_g: Glide::default(),
            gain_g: Glide::default(),
            out_g: Glide::default(),
            d: 1.0,
            modu: 1.0,
            osc: Phasor::default(),
            ch: [Channel::default(); 2],
        };
        r.set_sample_rate(48000.0);
        r.reset();
        r
    }

    /// `sr` is the OVERSAMPLED rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [&mut self.gain, &mut self.dist, &mut self.modulation, &mut self.output, &mut self.freq] {
            sm.set_time(sr, 20.0);
        }
        for c in self.ch.iter_mut() {
            c.dc.set_sample_rate(sr);
        }
        self.apply_block_values();
    }

    pub fn set(&mut self, s: RingSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.gain.set_target(s.gain_db);
        self.dist.set_target(s.dist);
        self.modulation.set_target(s.modulation);
        self.output.set_target(s.output_db);
        self.freq.set_target(s.freq_hz);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.gain.reset(self.s.gain_db);
        self.dist.reset(self.s.dist);
        self.modulation.reset(self.s.modulation);
        self.output.reset(self.s.output_db);
        self.freq.reset(self.s.freq_hz);
        self.clear_state();
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
        self.gain_g.hold(db_to_gain(self.gain.value()));
        self.out_g.hold(db_to_gain(self.output.value()));
    }

    fn clear_state(&mut self) {
        for c in self.ch.iter_mut() {
            c.dc.reset();
        }
        self.osc.reset();
    }

    fn apply_block_values(&mut self) {
        self.d = 1.0 + 9.0 * self.dist.value();
        self.modu = self.modulation.value();
        self.osc.set_freq(self.sr, self.freq.value());
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            self.clear_state();
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        let (g0, g1) = self.gain.block(frames);
        self.gain_g.set(db_to_gain(g0), db_to_gain(g1), frames);
        let (o0, o1) = self.output.block(frames);
        self.out_g.set(db_to_gain(o0), db_to_gain(o1), frames);
        self.dist.block(frames);
        self.modulation.block(frames);
        self.freq.block(frames);
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let g = self.gain_g.step();
        let o = self.out_g.step();
        let ph = self.osc.step();
        if m == 0.0 {
            return (l, r);
        }
        let osc = (TAU * ph).sin();
        let mode = self.s.mode;
        let yl = self.ch[0].tick(l, g, osc, mode, self.modu, self.d) * o;
        let yr = self.ch[1].tick(r, g, osc, mode, self.modu, self.d) * o;
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: RingSettings) -> Ring {
        let mut r = Ring::new();
        r.set_sample_rate(SR);
        r.set(s);
        r.reset();
        r
    }

    fn run(r: &mut Ring, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            r.begin_block(chunk.len());
            for &v in chunk {
                out.push(r.process_frame(v, v).0);
            }
        }
        out
    }

    fn clean(mode: RingMode, modulation: f32) -> RingSettings {
        RingSettings { on: true, gain_db: 0.0, dist: 0.0, modulation, output_db: 0.0, mode, freq_hz: 440.0 }
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut r = make(RingSettings { on: false, ..Default::default() });
        let x = noise(1000, 0.5, 21);
        assert_eq!(run(&mut r, &x), x);
    }

    #[test]
    fn osc_mode_makes_sum_and_difference_tones() {
        let mut r = make(clean(RingMode::Osc, 1.0));
        let y = run(&mut r, &sine(1000.0, SR, 52800, 0.3));
        let w = &y[4800..52800];
        assert!(tone_amplitude(w, 560.0, SR) > 0.08);
        assert!(tone_amplitude(w, 1440.0, SR) > 0.08);
        assert!(tone_amplitude(w, 1000.0, SR) < 0.01);
    }

    #[test]
    fn mod_zero_keeps_the_dry_note() {
        let mut r = make(clean(RingMode::Osc, 0.0));
        let y = run(&mut r, &sine(1000.0, SR, 52800, 0.3));
        let w = &y[4800..52800];
        assert!(tone_amplitude(w, 1000.0, SR) > 0.2);
        assert!(tone_amplitude(w, 560.0, SR) < 0.01);
    }

    #[test]
    fn self_mode_is_loud_and_dc_free() {
        let mut r = make(RingSettings { on: true, mode: RingMode::SelfMod, ..Default::default() });
        let y = run(&mut r, &sine(200.0, SR, 48000, 0.3));
        let tail = &y[24000..];
        let mean = tail.iter().sum::<f32>() / tail.len() as f32;
        assert!(rms(tail) > 0.01, "{}", rms(tail));
        assert!(mean.abs() < 0.01, "{mean}");
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 1.0, 22);
        for mode in RingMode::ALL {
            for (gain, dist, out) in [(0.0, 0.0, -24.0), (30.0, 1.0, 12.0)] {
                let mut r = make(RingSettings { on: true, gain_db: gain, dist, modulation: 1.0, output_db: out, mode, freq_hz: 2000.0 });
                let y = run(&mut r, &x);
                assert!(all_finite(&y));
                assert!(peak(&y) < 10.0, "{}", peak(&y));
            }
        }
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `41 passed`.

**Commit:** `S09: ring modulator module`

---

### S10 — `src/dsp/vibe.rs`

**Design (read only):**

| Stage | What it does |
|---|---|
| All-pass chain | 4 first-order stages with base frequencies 180, 520, 1400 and 3600 Hz. |
| LFO | `s = 0.5+0.5·sin`; sweep factor `2^(4·depth·(s²−0.5))`, i.e. up to ±2 octaves. The squared shape makes it dwell at the low end, like a lamp. |
| Stereo | The right LFO runs 90° (a quarter cycle) ahead. |
| Feedback | The previous output of stage 4 is added to the input (0…0.9). |
| Chorus mode | `(dry·dg + wet·wg)/(dg+wg)`, with the pedal mix law `dg = min(1, 2(1−mix))`, `wg = min(1, 2·mix)`. |
| Vibrato mode | 100 % wet; Mix is ignored. |

**Files:**
- Create `src/dsp/vibe.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod vibe;` below `pub mod ring;`.

`src/dsp/vibe.rs`
```rust
//! Vibe: four first-order all-pass stages swept by a lamp-like LFO, with feedback.

use crate::dsp::util::{mix, Glide, Phasor, Smoother, Switch};
use crate::params::VibeMode;
use std::f32::consts::{PI, TAU};

const BASES: [f32; 4] = [180.0, 520.0, 1400.0, 3600.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VibeSettings {
    pub on: bool,
    pub rate_hz: f32,
    pub depth: f32,
    pub feedback: f32,
    pub mix: f32,
    pub mode: VibeMode,
}

impl Default for VibeSettings {
    fn default() -> Self {
        Self { on: true, rate_hz: 1.5, depth: 0.6, feedback: 0.3, mix: 0.5, mode: VibeMode::Chorus }
    }
}

/// Pedal-style mix law: 0 = all dry, 0.5 = both full, 1 = all wet. Returns (dry gain, wet gain).
pub fn mix_gains(amount: f32) -> (f32, f32) {
    ((2.0 * (1.0 - amount)).min(1.0), (2.0 * amount).min(1.0))
}

/// LFO phase -> sweep factor (lamp-like: dwells at the low end).
#[inline]
fn sweep(phase: f32, depth: f32) -> f32 {
    let s = 0.5 + 0.5 * (TAU * phase).sin();
    2.0f32.powf(4.0 * depth * (s * s - 0.5))
}

#[derive(Clone, Copy, Default)]
struct Channel {
    x1: [f32; 4],
    y1: [f32; 4],
    last: f32,
    lfo: Phasor,
}

impl Channel {
    fn clear(&mut self) {
        self.x1 = [0.0; 4];
        self.y1 = [0.0; 4];
        self.last = 0.0;
    }

    #[inline]
    fn tick(&mut self, x: f32, factor: f32, fb: f32, sr: f32) -> f32 {
        let mut u = x + fb * self.last;
        for k in 0..4 {
            let f = (BASES[k] * factor).clamp(20.0, 0.45 * sr);
            let t = (PI * f / sr).tan();
            let a = (t - 1.0) / (t + 1.0);
            let y = a * u + self.x1[k] - a * self.y1[k];
            self.x1[k] = u;
            self.y1[k] = y;
            u = y;
        }
        self.last = u;
        u
    }
}

pub struct Vibe {
    sr: f32,
    s: VibeSettings,
    switch: Switch,
    depth: Smoother,
    feedback: Smoother,
    mix_sm: Smoother,
    mix_g: Glide,
    depth_v: f32,
    fb_v: f32,
    dg: f32,
    wg: f32,
    ch: [Channel; 2],
}

impl Vibe {
    pub fn new() -> Self {
        let s = VibeSettings::default();
        let mut v = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            depth: Smoother::new(s.depth),
            feedback: Smoother::new(s.feedback),
            mix_sm: Smoother::new(s.mix),
            mix_g: Glide::default(),
            depth_v: s.depth,
            fb_v: s.feedback,
            dg: 1.0,
            wg: 1.0,
            ch: [Channel::default(); 2],
        };
        v.set_sample_rate(48000.0);
        v.reset();
        v
    }

    /// Host rate. Does not allocate.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        for sm in [&mut self.depth, &mut self.feedback, &mut self.mix_sm] {
            sm.set_time(sr, 20.0);
        }
        self.apply_block_values();
    }

    pub fn set(&mut self, s: VibeSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.depth.set_target(s.depth);
        self.feedback.set_target(s.feedback);
        self.mix_sm.set_target(s.mix);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.depth.reset(self.s.depth);
        self.feedback.reset(self.s.feedback);
        self.mix_sm.reset(self.s.mix);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.ch[0].lfo.set_phase(0.0);
        self.ch[1].lfo.set_phase(0.25);
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
    }

    fn apply_block_values(&mut self) {
        self.depth_v = self.depth.value();
        self.fb_v = self.feedback.value();
        let (dg, wg) = mix_gains(self.mix_sm.value());
        self.dg = dg;
        self.wg = wg;
        let (sr, rate) = (self.sr, self.s.rate_hz);
        for c in self.ch.iter_mut() {
            c.lfo.set_freq(sr, rate);
        }
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        self.depth.block(frames);
        self.feedback.block(frames);
        self.mix_sm.block(frames);
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        let pl = self.ch[0].lfo.step();
        let pr = self.ch[1].lfo.step();
        if m == 0.0 {
            return (l, r);
        }
        let sr = self.sr;
        let wl = self.ch[0].tick(l, sweep(pl, self.depth_v), self.fb_v, sr);
        let wr = self.ch[1].tick(r, sweep(pr, self.depth_v), self.fb_v, sr);
        let (out_l, out_r) = match self.s.mode {
            VibeMode::Vibrato => (wl, wr),
            VibeMode::Chorus => {
                let norm = self.dg + self.wg;
                ((l * self.dg + wl * self.wg) / norm, (r * self.dg + wr * self.wg) / norm)
            }
        };
        (mix(l, out_l, m), mix(r, out_r, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: VibeSettings) -> Vibe {
        let mut v = Vibe::new();
        v.set_sample_rate(SR);
        v.set(s);
        v.reset();
        v
    }

    fn run(v: &mut Vibe, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            v.begin_block(chunk.len());
            for &s in chunk {
                out.push(v.process_frame(s, s).0);
            }
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut v = make(VibeSettings { on: false, ..Default::default() });
        let x = noise(1000, 0.5, 31);
        assert_eq!(run(&mut v, &x), x);
    }

    #[test]
    fn chorus_with_mix_zero_is_dry() {
        let mut v = make(VibeSettings { mix: 0.0, ..Default::default() });
        let x = noise(4800, 0.5, 32);
        assert_eq!(run(&mut v, &x), x);
    }

    #[test]
    fn vibrato_without_depth_is_an_allpass() {
        let mut v = make(VibeSettings { mode: VibeMode::Vibrato, depth: 0.0, feedback: 0.0, ..Default::default() });
        let x = sine(1000.0, SR, 9600, 0.5);
        let y = run(&mut v, &x);
        let ratio = rms(&y[4800..]) / rms(&x[4800..]);
        assert!((ratio - 1.0).abs() < 0.01, "{ratio}");
    }

    #[test]
    fn lfo_sweeps_the_notches() {
        let mut v = make(VibeSettings { on: true, rate_hz: 2.0, depth: 1.0, feedback: 0.0, mix: 0.5, mode: VibeMode::Chorus });
        let x = sine(1000.0, SR, 48000, 0.5);
        let y = run(&mut v, &x);
        let windows: Vec<f32> = y[4800..].chunks(480).map(rms).collect();
        let max = windows.iter().cloned().fold(0.0f32, f32::max);
        let min = windows.iter().cloned().fold(f32::MAX, f32::min);
        assert!(max > 2.0 * min, "max {max}, min {min}");
    }

    #[test]
    fn extremes_stay_finite() {
        let x = noise(9600, 0.5, 33);
        for mode in VibeMode::ALL {
            let mut v = make(VibeSettings { on: true, rate_hz: 10.0, depth: 1.0, feedback: 0.9, mix: 1.0, mode });
            let y = run(&mut v, &x);
            assert!(all_finite(&y));
            assert!(peak(&y) < 10.0, "{}", peak(&y));
        }
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `46 passed`.

**Commit:** `S10: vibe module`

---

### S11 — `src/dsp/prism.rs`

**Design (read only):**

| Stage | What it does |
|---|---|
| Shifter | Two read heads per voice on one delay line. Phase `φ += (ratio−1)/grain`; delay `= 2 + extra + grain·(1−φ)`. The second head is at `φ+0.5`. Hann crossfade `sin²(πφ)`, so the two weights always sum to 1. |
| Voices | Primary ratio `2^(pitch/12)`. Secondary ratio is twice that (one octave above). |
| Tracking | Grain length 10–120 ms. |
| Depth | Read-head modulation of up to 4 ms at 0.7 Hz. The right LFO is a quarter cycle ahead. |
| Halo | `tanh(halo · last shifted output)` is added to the delay-line input. |
| Tone | 2-pole low-pass on each voice. |
| Output | `dry·(1−primary) + primary·voice1 + secondary·voice2`. |
| Memory | `prepare()` allocates 0.25 s per channel. |

> **NOTE** — A granular shifter does not put its energy at exactly 2× the input frequency. It spreads it over lines spaced at the grain rate around that frequency. That is why the tests measure energy in a ±60 Hz band.

**Files:**
- Create `src/dsp/prism.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod prism;` below `pub mod vibe;`.

`src/dsp/prism.rs`
```rust
//! Prism (after the Rainbow Machine): dual-head delay-line pitch shifter with two voices,
//! "halo" feedback through the shifter, and a tone low-pass on the wet path.

use crate::dsp::util::{mix, Biquad, DelayLine, Glide, Phasor, Smoother, Switch};
use std::f32::consts::{FRAC_1_SQRT_2, PI, TAU};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrismSettings {
    pub on: bool,
    pub pitch_st: f32,
    pub primary: f32,
    pub secondary: f32,
    pub tracking_ms: f32,
    pub depth: f32,
    pub halo: f32,
    pub tone_hz: f32,
}

impl Default for PrismSettings {
    fn default() -> Self {
        Self {
            on: false,
            pitch_st: 12.0,
            primary: 0.5,
            secondary: 0.0,
            tracking_ms: 40.0,
            depth: 0.2,
            halo: 0.0,
            tone_hz: 8000.0,
        }
    }
}

/// Per-sub-block constants shared by both channels.
#[derive(Clone, Copy, Debug, Default)]
struct Block {
    r1: f32,
    r2: f32,
    grain: f32,
    extra: f32,
    primary: f32,
    secondary: f32,
    halo: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Head {
    phase: f32,
}

impl Head {
    #[inline]
    fn read(&mut self, line: &DelayLine, ratio: f32, grain: f32, extra: f32) -> f32 {
        self.phase = (self.phase + (ratio - 1.0) / grain).rem_euclid(1.0);
        let p1 = self.phase;
        let mut p2 = p1 + 0.5;
        if p2 >= 1.0 {
            p2 -= 1.0;
        }
        let w = (PI * p1).sin();
        let w1 = w * w;
        let d1 = 2.0 + extra + grain * (1.0 - p1);
        let d2 = 2.0 + extra + grain * (1.0 - p2);
        line.read(d1) * w1 + line.read(d2) * (1.0 - w1)
    }
}

struct Channel {
    line: DelayLine,
    v1: Head,
    v2: Head,
    t1: Biquad,
    t2: Biquad,
    last: f32,
    lfo: Phasor,
}

impl Channel {
    fn new() -> Self {
        Self {
            line: DelayLine::new(),
            v1: Head::default(),
            v2: Head::default(),
            t1: Biquad::default(),
            t2: Biquad::default(),
            last: 0.0,
            lfo: Phasor::default(),
        }
    }

    fn clear(&mut self) {
        self.line.reset();
        self.v1 = Head::default();
        self.v2 = Head::default();
        self.t1.reset();
        self.t2.reset();
        self.last = 0.0;
    }

    #[inline]
    fn tick(&mut self, x: f32, k: &Block) -> f32 {
        let fb = (k.halo * self.last).tanh();
        self.line.push(x + fb);
        let lfo = 0.5 + 0.5 * (TAU * self.lfo.step()).sin();
        let extra = k.extra * lfo;
        let a = self.t1.tick(self.v1.read(&self.line, k.r1, k.grain, extra));
        let b = self.t2.tick(self.v2.read(&self.line, k.r2, k.grain, extra));
        self.last = a + k.secondary * b;
        x * (1.0 - k.primary) + a * k.primary + b * k.secondary
    }
}

pub struct Prism {
    sr: f32,
    s: PrismSettings,
    switch: Switch,
    pitch: Smoother,
    primary: Smoother,
    secondary: Smoother,
    tracking: Smoother,
    depth: Smoother,
    halo: Smoother,
    tone: Smoother,
    mix_g: Glide,
    k: Block,
    ch: [Channel; 2],
}

impl Prism {
    /// Does not allocate the delay lines; call `prepare` before use.
    pub fn new() -> Self {
        let s = PrismSettings::default();
        Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            pitch: Smoother::new(s.pitch_st),
            primary: Smoother::new(s.primary),
            secondary: Smoother::new(s.secondary),
            tracking: Smoother::new(s.tracking_ms),
            depth: Smoother::new(s.depth),
            halo: Smoother::new(s.halo),
            tone: Smoother::new(s.tone_hz),
            mix_g: Glide::default(),
            k: Block::default(),
            ch: [Channel::new(), Channel::new()],
        }
    }

    /// Allocates 0.25 s of delay per channel. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        self.sr = sr;
        let cap = (0.25 * sr) as usize + 8;
        for c in self.ch.iter_mut() {
            c.line.allocate(cap);
            c.lfo.set_freq(sr, 0.7);
        }
        self.switch.set_sample_rate(sr);
        for sm in [
            &mut self.pitch,
            &mut self.primary,
            &mut self.secondary,
            &mut self.tracking,
            &mut self.depth,
            &mut self.halo,
            &mut self.tone,
        ] {
            sm.set_time(sr, 20.0);
        }
        self.reset();
    }

    pub fn set(&mut self, s: PrismSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.pitch.set_target(s.pitch_st);
        self.primary.set_target(s.primary);
        self.secondary.set_target(s.secondary);
        self.tracking.set_target(s.tracking_ms);
        self.depth.set_target(s.depth);
        self.halo.set_target(s.halo);
        self.tone.set_target(s.tone_hz);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.pitch.reset(self.s.pitch_st);
        self.primary.reset(self.s.primary);
        self.secondary.reset(self.s.secondary);
        self.tracking.reset(self.s.tracking_ms);
        self.depth.reset(self.s.depth);
        self.halo.reset(self.s.halo);
        self.tone.reset(self.s.tone_hz);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.ch[0].lfo.set_phase(0.0);
        self.ch[1].lfo.set_phase(0.25);
        self.apply_block_values();
        self.mix_g.hold(self.switch.value());
    }

    fn apply_block_values(&mut self) {
        let sr = self.sr;
        self.k.r1 = 2.0f32.powf(self.pitch.value() / 12.0);
        self.k.r2 = self.k.r1 * 2.0;
        self.k.grain = (self.tracking.value() * 0.001 * sr).max(16.0);
        self.k.extra = self.depth.value() * 0.004 * sr;
        self.k.primary = self.primary.value();
        self.k.secondary = self.secondary.value();
        self.k.halo = self.halo.value();
        let tone = self.tone.value();
        for c in self.ch.iter_mut() {
            c.t1.set_lowpass(sr, tone, FRAC_1_SQRT_2);
            c.t2.set_lowpass(sr, tone, FRAC_1_SQRT_2);
        }
    }

    pub fn is_off(&self) -> bool {
        self.switch.is_off()
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.mix_g.set(m0, m1, frames);
        for sm in [
            &mut self.pitch,
            &mut self.primary,
            &mut self.secondary,
            &mut self.tracking,
            &mut self.depth,
            &mut self.halo,
            &mut self.tone,
        ] {
            sm.block(frames);
        }
        self.apply_block_values();
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.mix_g.step();
        if m == 0.0 {
            return (l, r);
        }
        let k = self.k;
        let yl = self.ch[0].tick(l, &k);
        let yr = self.ch[1].tick(r, &k);
        (mix(l, yl, m), mix(r, yr, m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn make(s: PrismSettings) -> Prism {
        let mut p = Prism::new();
        p.prepare(SR);
        p.set(s);
        p.reset();
        p
    }

    fn run(p: &mut Prism, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            p.begin_block(chunk.len());
            for &v in chunk {
                out.push(p.process_frame(v, v).0);
            }
        }
        out
    }

    /// Amplitude of everything within +-60 Hz of `center` (window must be exactly 1 s).
    fn band(x: &[f32], center: f32) -> f32 {
        let mut e = 0.0f32;
        let mut f = center - 60.0;
        while f <= center + 60.0 {
            let a = tone_amplitude(x, f, SR);
            e += a * a;
            f += 1.0;
        }
        e.sqrt()
    }

    fn clean(pitch_st: f32, primary: f32, secondary: f32) -> PrismSettings {
        PrismSettings { on: true, pitch_st, primary, secondary, tracking_ms: 40.0, depth: 0.0, halo: 0.0, tone_hz: 16000.0 }
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut p = make(PrismSettings { on: false, ..Default::default() });
        let x = noise(1000, 0.5, 41);
        assert_eq!(run(&mut p, &x), x);
    }

    #[test]
    fn plus_12_moves_energy_an_octave_up() {
        let mut p = make(clean(12.0, 1.0, 0.0));
        let y = run(&mut p, &sine(440.0, SR, 57600, 0.5));
        let w = &y[9600..57600];
        let up = band(w, 880.0);
        let orig = band(w, 440.0);
        assert!(up > 0.2, "{up}");
        assert!(up > 4.0 * orig, "up {up}, orig {orig}");
    }

    #[test]
    fn minus_12_moves_energy_an_octave_down() {
        let mut p = make(clean(-12.0, 1.0, 0.0));
        let y = run(&mut p, &sine(440.0, SR, 57600, 0.5));
        let w = &y[9600..57600];
        let down = band(w, 220.0);
        let orig = band(w, 440.0);
        assert!(down > 0.2, "{down}");
        assert!(down > 4.0 * orig, "down {down}, orig {orig}");
    }

    #[test]
    fn secondary_adds_an_octave_above_primary() {
        let mut p = make(clean(0.0, 1.0, 1.0));
        let y = run(&mut p, &sine(440.0, SR, 57600, 0.5));
        let w = &y[9600..57600];
        assert!(band(w, 880.0) > 0.15, "{}", band(w, 880.0));
        assert!(band(w, 440.0) > 0.15, "{}", band(w, 440.0));
    }

    #[test]
    fn halo_extremes_stay_finite_and_bounded() {
        let x = noise(19200, 0.5, 42);
        for tracking in [10.0, 120.0] {
            let mut p = make(PrismSettings {
                on: true,
                pitch_st: 12.0,
                primary: 1.0,
                secondary: 1.0,
                tracking_ms: tracking,
                depth: 1.0,
                halo: 0.95,
                tone_hz: 16000.0,
            });
            let y = run(&mut p, &x);
            assert!(all_finite(&y));
            assert!(peak(&y) < 5.0, "{}", peak(&y));
        }
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `51 passed`.

> **NOTE** — The Prism tests take a few seconds in debug builds. That is normal.

**Commit:** `S11: prism pitch shifter module`

---

### S12 — `src/dsp/echo.rs`

**Design (read only):**

| Item | What it does |
|---|---|
| Delay line | 6.2 s per channel, allocated in `prepare()`. Reverse mode reads up to 2× Time. |
| Time | 10–3000 ms. Smoothed over 150 ms, so turning it glides like tape. With Sync on: `60000/bpm · beats` (unknown tempo → 120 bpm). |
| Read and write order | Read BEFORE writing, at `delay − 1`, so the first repeat lands exactly `Time` after the input. |
| Tone | One-pole low-pass on every repeat, including the first. Tape mode uses 0.6 × the cutoff. |
| Feedback | `tanh(feedback · repeat)` is written back; Tape uses `tanh(1.5v)/1.5`. Up to 110 % self-oscillates safely. |
| Modulation | Digital, Multi and Reverse: 0.8 Hz sine, up to 1 ms. Tape: 0.5 Hz wow (4 ms) + 6 Hz flutter (0.4 ms) + filtered noise (0.3 ms). The offset only ever adds delay. |
| Multi | Taps at ⅓, ⅔ and 1 × Time, weighted `0.6·(0.5, 0.75, 1.0)`. |
| Reverse | Two heads read backwards through slices of length Time, with a Hann crossfade. |
| Ping-pong | The mono input goes to the left line; left feeds right and right feeds left. |
| Mix | Pedal law `mix_gains` from vibe.rs: dry stays at 1 up to 50 %. |
| Bypass | The input is faded out of the lines. Trails on: repeats keep sounding. Trails off: repeats fade with the switch and the module goes silent (dry only). |

**Files:**
- Create `src/dsp/echo.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod echo;` below `pub mod prism;`.

`src/dsp/echo.rs`
```rust
//! Echo (after the Stereo Memory Man): stereo delay up to 3 s with a soft limiter in the
//! feedback loop, Digital / Tape / Multi / Reverse, tempo sync, ping-pong and trails.

use crate::dsp::util::{DelayLine, Glide, OnePole, Phasor, Rng, Smoother, Switch};
use crate::dsp::vibe::mix_gains;
use crate::params::{Division, EchoMode};
use std::f32::consts::{PI, TAU};

/// Reverse mode reads up to 2 x Time (3 s), plus modulation headroom.
const MAX_SECONDS: f32 = 6.2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EchoSettings {
    pub on: bool,
    pub time_ms: f32,
    pub sync: bool,
    pub division: Division,
    pub feedback: f32,
    pub mix: f32,
    pub tone_hz: f32,
    pub modulation: f32,
    pub mode: EchoMode,
    pub pingpong: bool,
    pub trails: bool,
    /// Host tempo in BPM; 0 or less means unknown (120 is used).
    pub tempo_bpm: f32,
}

impl Default for EchoSettings {
    fn default() -> Self {
        Self {
            on: true,
            time_ms: 400.0,
            sync: false,
            division: Division::Quarter,
            feedback: 0.35,
            mix: 0.35,
            tone_hz: 6000.0,
            modulation: 0.1,
            mode: EchoMode::Digital,
            pingpong: false,
            trails: true,
            tempo_bpm: 120.0,
        }
    }
}

impl EchoSettings {
    /// Delay time in ms after applying Sync (plan.md C1a).
    pub fn effective_time_ms(&self) -> f32 {
        if self.sync {
            let bpm = if self.tempo_bpm > 1.0 { self.tempo_bpm } else { 120.0 };
            (60000.0 / bpm * self.division.beats()).clamp(10.0, 3000.0)
        } else {
            self.time_ms.clamp(10.0, 3000.0)
        }
    }
}

struct Channel {
    line: DelayLine,
    tone: OnePole,
    rev_phase: f32,
}

impl Channel {
    fn new() -> Self {
        Self { line: DelayLine::new(), tone: OnePole::default(), rev_phase: 0.0 }
    }

    fn clear(&mut self) {
        self.line.reset();
        self.tone.reset();
        self.rev_phase = 0.0;
    }

    /// Reads the delayed signal BEFORE this frame's write. `d` = delay in samples.
    #[inline]
    fn read(&mut self, mode: EchoMode, d: f32) -> f32 {
        match mode {
            EchoMode::Multi => {
                let t1 = self.line.read(d / 3.0 - 1.0);
                let t2 = self.line.read(d * 2.0 / 3.0 - 1.0);
                let t3 = self.line.read(d - 1.0);
                0.6 * (0.5 * t1 + 0.75 * t2 + t3)
            }
            EchoMode::Reverse => {
                self.rev_phase += 1.0 / d.max(1.0);
                if self.rev_phase >= 1.0 {
                    self.rev_phase -= 1.0;
                }
                let p1 = self.rev_phase;
                let mut p2 = p1 + 0.5;
                if p2 >= 1.0 {
                    p2 -= 1.0;
                }
                let w = (PI * p1).sin();
                let w1 = w * w;
                self.line.read(2.0 * d * p1) * w1 + self.line.read(2.0 * d * p2) * (1.0 - w1)
            }
            EchoMode::Digital | EchoMode::Tape => self.line.read(d - 1.0),
        }
    }
}

pub struct Echo {
    sr: f32,
    s: EchoSettings,
    switch: Switch,
    time: Smoother,
    feedback: Smoother,
    mix_sm: Smoother,
    tone: Smoother,
    modulation_sm: Smoother,
    sw_g: Glide,
    time_g: Glide,
    fb: f32,
    dg: f32,
    wg: f32,
    modulation: f32,
    wow: Phasor,
    flutter: Phasor,
    noise_lp: OnePole,
    rng: Rng,
    ch: [Channel; 2],
}

impl Echo {
    /// Does not allocate the delay lines; call `prepare` before use.
    pub fn new() -> Self {
        let s = EchoSettings::default();
        let mut e = Self {
            sr: 48000.0,
            s,
            switch: Switch::new(s.on),
            time: Smoother::new(s.effective_time_ms()),
            feedback: Smoother::new(s.feedback),
            mix_sm: Smoother::new(s.mix),
            tone: Smoother::new(s.tone_hz),
            modulation_sm: Smoother::new(s.modulation),
            sw_g: Glide::default(),
            time_g: Glide::default(),
            fb: s.feedback,
            dg: 1.0,
            wg: 0.7,
            modulation: s.modulation,
            wow: Phasor::default(),
            flutter: Phasor::default(),
            noise_lp: OnePole::default(),
            rng: Rng::new(0x1234_5678),
            ch: [Channel::new(), Channel::new()],
        };
        e.set_rates(48000.0);
        e.reset();
        e
    }

    fn set_rates(&mut self, sr: f32) {
        self.sr = sr;
        self.switch.set_sample_rate(sr);
        self.time.set_time(sr, 150.0);
        for sm in [&mut self.feedback, &mut self.mix_sm, &mut self.tone, &mut self.modulation_sm] {
            sm.set_time(sr, 20.0);
        }
        self.flutter.set_freq(sr, 6.0);
        self.noise_lp.set_cutoff(sr, 20.0);
    }

    /// Allocates 6.2 s of delay per channel. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        let cap = (MAX_SECONDS * sr) as usize + 8;
        for c in self.ch.iter_mut() {
            c.line.allocate(cap);
        }
        self.set_rates(sr);
        self.reset();
    }

    pub fn set(&mut self, s: EchoSettings) {
        self.s = s;
        self.switch.set(s.on);
        self.time.set_target(s.effective_time_ms());
        self.feedback.set_target(s.feedback);
        self.mix_sm.set_target(s.mix);
        self.tone.set_target(s.tone_hz);
        self.modulation_sm.set_target(s.modulation);
    }

    pub fn reset(&mut self) {
        self.switch.reset(self.s.on);
        self.time.reset(self.s.effective_time_ms());
        self.feedback.reset(self.s.feedback);
        self.mix_sm.reset(self.s.mix);
        self.tone.reset(self.s.tone_hz);
        self.modulation_sm.reset(self.s.modulation);
        for c in self.ch.iter_mut() {
            c.clear();
        }
        self.wow.reset();
        self.flutter.reset();
        self.noise_lp.reset();
        self.apply_block_values();
        self.time_g.hold(self.time.value() * self.sr / 1000.0);
        self.sw_g.hold(self.switch.value());
    }

    fn apply_block_values(&mut self) {
        self.fb = self.feedback.value();
        let (dg, wg) = mix_gains(self.mix_sm.value());
        self.dg = dg;
        self.wg = wg;
        self.modulation = self.modulation_sm.value();
        let tape = self.s.mode == EchoMode::Tape;
        let tone = if tape { self.tone.value() * 0.6 } else { self.tone.value() };
        let sr = self.sr;
        for c in self.ch.iter_mut() {
            c.tone.set_cutoff(sr, tone);
        }
        self.wow.set_freq(sr, if tape { 0.5 } else { 0.8 });
    }

    /// False while trails are still allowed to ring.
    pub fn is_off(&self) -> bool {
        self.switch.is_off() && !self.s.trails
    }

    pub fn begin_block(&mut self, frames: usize) {
        if self.switch.value() == 0.0 && self.s.on && !self.s.trails {
            for c in self.ch.iter_mut() {
                c.clear();
            }
        }
        let (m0, m1) = self.switch.block(frames);
        self.sw_g.set(m0, m1, frames);
        let (t0, t1) = self.time.block(frames);
        let k = self.sr / 1000.0;
        self.time_g.set(t0 * k, t1 * k, frames);
        self.feedback.block(frames);
        self.mix_sm.block(frames);
        self.tone.block(frames);
        self.modulation_sm.block(frames);
        self.apply_block_values();
    }

    /// Extra delay in samples from wow/flutter. Always >= 0.
    #[inline]
    fn mod_offset(&mut self) -> f32 {
        let wow = 0.5 + 0.5 * (TAU * self.wow.step()).sin();
        let flutter = 0.5 + 0.5 * (TAU * self.flutter.step()).sin();
        let n = self.noise_lp.lowpass(self.rng.next_f32()).abs();
        let amt = self.modulation * self.sr;
        if self.s.mode == EchoMode::Tape {
            amt * (0.004 * wow + 0.0004 * flutter + 0.0003 * n)
        } else {
            amt * 0.001 * wow
        }
    }

    #[inline]
    fn saturate(&self, v: f32) -> f32 {
        if self.s.mode == EchoMode::Tape {
            (1.5 * v).tanh() / 1.5
        } else {
            v.tanh()
        }
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = self.sw_g.step();
        let base = self.time_g.step();
        let off = self.mod_offset();
        if m == 0.0 && !self.s.trails {
            return (l, r);
        }
        let d = base + off;
        let mode = self.s.mode;
        let yl = self.ch[0].read(mode, d);
        let yr = self.ch[1].read(mode, d);
        let wl = self.ch[0].tone.lowpass(yl);
        let wr = self.ch[1].tone.lowpass(yr);
        let fl = self.saturate(self.fb * wl);
        let fr = self.saturate(self.fb * wr);
        let (il, ir) = (l * m, r * m);
        let (inl, inr) = if self.s.pingpong { ((il + ir) * 0.5 + fr, fl) } else { (il + fl, ir + fr) };
        self.ch[0].line.push(inl);
        self.ch[1].line.push(inr);
        let dry = 1.0 + (self.dg - 1.0) * m;
        let wet = if self.s.trails { self.wg } else { self.wg * m };
        (l * dry + wl * wet, r * dry + wr * wet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 48000.0;

    fn base() -> EchoSettings {
        EchoSettings {
            on: true,
            time_ms: 400.0,
            sync: false,
            division: Division::Quarter,
            feedback: 0.0,
            mix: 1.0,
            tone_hz: 16000.0,
            modulation: 0.0,
            mode: EchoMode::Digital,
            pingpong: false,
            trails: false,
            tempo_bpm: 120.0,
        }
    }

    fn make(s: EchoSettings) -> Echo {
        let mut e = Echo::new();
        e.prepare(SR);
        e.set(s);
        e.reset();
        e
    }

    fn run(e: &mut Echo, l: &[f32], r: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut out_l = Vec::with_capacity(l.len());
        let mut out_r = Vec::with_capacity(r.len());
        let mut i = 0;
        while i < l.len() {
            let n = SUB.min(l.len() - i);
            e.begin_block(n);
            for k in i..i + n {
                let (a, b) = e.process_frame(l[k], r[k]);
                out_l.push(a);
                out_r.push(b);
            }
            i += n;
        }
        (out_l, out_r)
    }

    fn burst(len: usize, on: usize, freq: f32, amp: f32) -> Vec<f32> {
        let mut x = sine(freq, SR, len, amp);
        for v in x.iter_mut().skip(on) {
            *v = 0.0;
        }
        x
    }

    fn run_switch_off(s: EchoSettings, at: usize, x: &[f32]) -> Vec<f32> {
        let mut e = make(s);
        let mut out = Vec::with_capacity(x.len());
        let mut done = 0;
        for chunk in x.chunks(SUB) {
            if done >= at {
                e.set(EchoSettings { on: false, ..s });
            }
            e.begin_block(chunk.len());
            for &v in chunk {
                out.push(e.process_frame(v, v).0);
            }
            done += chunk.len();
        }
        out
    }

    #[test]
    fn off_is_exact_passthrough() {
        let mut e = make(EchoSettings { on: false, trails: false, ..base() });
        let x = noise(4800, 0.5, 51);
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(l, x);
    }

    #[test]
    fn first_repeat_arrives_after_time() {
        let mut e = make(base());
        let mut x = vec![0.0f32; 24000];
        x[0] = 1.0;
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(argmax_abs(&l), 19200);
    }

    #[test]
    fn sync_uses_host_tempo() {
        let mut e = make(EchoSettings { sync: true, tempo_bpm: 120.0, division: Division::Quarter, ..base() });
        let mut x = vec![0.0f32; 30000];
        x[0] = 1.0;
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(argmax_abs(&l), 24000);
    }

    #[test]
    fn feedback_sets_repeat_level() {
        let mut e = make(EchoSettings { time_ms: 200.0, feedback: 0.5, ..base() });
        let x = burst(21000, 960, 500.0, 0.1);
        let (l, _) = run(&mut e, &x, &x);
        let first = rms(&l[9600..10560]);
        let second = rms(&l[19200..20160]);
        let ratio = second / first;
        assert!(first > 0.05, "{first}");
        assert!(ratio > 0.4 && ratio < 0.6, "{ratio}");
    }

    #[test]
    fn feedback_above_100_percent_self_oscillates_safely() {
        let mut e = make(EchoSettings { time_ms: 100.0, feedback: 1.1, tone_hz: 6000.0, ..base() });
        let mut x = noise(144000, 0.5, 52);
        for v in x.iter_mut().skip(4800) {
            *v = 0.0;
        }
        let (l, r) = run(&mut e, &x, &x);
        assert!(all_finite(&l) && all_finite(&r));
        assert!(peak(&l) < 3.0, "{}", peak(&l));
        assert!(rms(&l[120000..]) > 0.01, "{}", rms(&l[120000..]));
    }

    #[test]
    fn pingpong_alternates_sides() {
        let mut e = make(EchoSettings { time_ms: 100.0, feedback: 0.5, pingpong: true, ..base() });
        let mut x = vec![0.0f32; 12000];
        x[0] = 1.0;
        let (l, r) = run(&mut e, &x, &x);
        assert!(peak(&l[4790..4810]) > 0.3);
        assert!(peak(&r[4790..4810]) < 0.01);
        assert!(peak(&r[9590..9610]) > 0.1);
        assert!(peak(&l[9590..9610]) < 0.01);
    }

    #[test]
    fn trails_ring_out_after_bypass() {
        let s = EchoSettings { time_ms: 100.0, feedback: 0.7, mix: 0.5, ..base() };
        let x = burst(48000, 960, 500.0, 0.3);
        let with = run_switch_off(EchoSettings { trails: true, ..s }, 2400, &x);
        let without = run_switch_off(EchoSettings { trails: false, ..s }, 2400, &x);
        assert!(rms(&with[24000..]) > 1e-4, "{}", rms(&with[24000..]));
        assert!(without[24000..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn multi_mode_has_three_taps() {
        let mut e = make(EchoSettings { time_ms: 300.0, mode: EchoMode::Multi, ..base() });
        let mut x = vec![0.0f32; 16000];
        x[0] = 1.0;
        let (l, _) = run(&mut e, &x, &x);
        assert_eq!(4000 + argmax_abs(&l[4000..5600]), 4800);
        assert_eq!(9000 + argmax_abs(&l[9000..10200]), 9600);
        assert_eq!(14000 + argmax_abs(&l[14000..15000]), 14400);
    }

    #[test]
    fn reverse_mode_produces_sound() {
        let mut e = make(EchoSettings { time_ms: 100.0, mode: EchoMode::Reverse, ..base() });
        let x = burst(24000, 2400, 500.0, 0.3);
        let (l, r) = run(&mut e, &x, &x);
        assert!(all_finite(&l) && all_finite(&r));
        assert!(rms(&l) > 0.01, "{}", rms(&l));
    }

    #[test]
    fn extremes_stay_finite_and_bounded() {
        let x = noise(9600, 0.5, 53);
        for mode in EchoMode::ALL {
            for (time, tone, pingpong) in [(10.0, 1000.0, false), (10.0, 16000.0, true), (3000.0, 16000.0, true)] {
                let mut e = make(EchoSettings {
                    time_ms: time,
                    feedback: 1.1,
                    modulation: 1.0,
                    tone_hz: tone,
                    mode,
                    pingpong,
                    trails: true,
                    mix: 0.5,
                    ..base()
                });
                let (l, r) = run(&mut e, &x, &x);
                assert!(all_finite(&l) && all_finite(&r), "{mode:?}");
                assert!(peak(&l) < 4.0 && peak(&r) < 4.0, "{mode:?} {time}: {}", peak(&l));
            }
        }
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `61 passed`.

> **WARNING** — Never raise the 1.1 feedback limit or remove the `tanh` in `saturate`. The limiter is what keeps a self-oscillating echo below full scale (Part A, WARNING class).

**Commit:** `S12: echo module`

<!-- END OF PART 3. PART 4 (Steps S13–S20: Looper, Tuner, chain, final lib.rs, fonts, editor skeleton, paint/layout, knob) IS APPENDED BELOW. -->
# PART D4 — STEPS S13 TO S20

## D4.0 Erratum to Part C (binding)

**C3, Uppercase column — corrected.** The reference CSS uppercases only `.label`, `.option` and the pedal name. The column therefore reads:

| Style | Uppercase |
|---|---|
| LABEL | yes |
| OPTION | yes |
| PEDAL_NAME | yes |
| WORDMARK | no |
| NOTE | no |
| VALUE | no |
| READOUT | no |
| STAGE | no |

Draw READOUT and STAGE text exactly as written, for example `MAX 30.0 s` and `A4 = 440 Hz`.

---

### S13 — `src/dsp/looper.rs`

**Design (read only):**

| Item | Rule |
|---|---|
| Buffer | 30 s stereo, allocated in `prepare()`. |
| Loop button | Empty→Record, Record→Play, Play→Overdub, Overdub→Play, Stopped→Play (from the start). |
| Stop button | Record→Stopped (keeps the loop). Play or Overdub→Stopped (rewinds to the start). |
| Clear button | Always goes to Empty. |
| Button edges | Acted on only on the rising edge (B10). MIDI calls `press_*` directly. |
| Recording | Auto-closes at 30 s and switches to Play. Closing a loop fades its first and last 96 samples. |
| Output | `input + loop × level`. Loop level −60 dB is silence. Reverse plays backwards. |
| `reset()` | Keeps the loop, stops playback. |

**Files:**
- Create `src/dsp/looper.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod looper;` below `pub mod echo;`.

`src/dsp/looper.rs`
```rust
//! Looper (after the Hazarai): 30 s stereo, one-button cycle Record -> Play -> Overdub -> Play.
//! Buttons act on their rising edge (plan.md B10). MIDI notes call `press_*` directly.

use crate::dsp::util::{db_to_gain, Glide, Smoother};

pub const MAX_LOOP_SECONDS: f32 = 30.0;
const FADE: usize = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopState {
    Empty,
    Record,
    Play,
    Overdub,
    Stopped,
}

impl LoopState {
    pub fn to_u8(self) -> u8 {
        match self {
            LoopState::Empty => 0,
            LoopState::Record => 1,
            LoopState::Play => 2,
            LoopState::Overdub => 3,
            LoopState::Stopped => 4,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => LoopState::Record,
            2 => LoopState::Play,
            3 => LoopState::Overdub,
            4 => LoopState::Stopped,
            _ => LoopState::Empty,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LooperSettings {
    pub level_db: f32,
    pub reverse: bool,
}

impl Default for LooperSettings {
    fn default() -> Self {
        Self { level_db: 0.0, reverse: false }
    }
}

/// Playback gain; -60 dB and below is silence.
pub fn level_gain(db: f32) -> f32 {
    if db <= -59.95 {
        0.0
    } else {
        db_to_gain(db)
    }
}

pub struct Looper {
    buf: [Vec<f32>; 2],
    cap: usize,
    len: usize,
    pos: usize,
    state: LoopState,
    s: LooperSettings,
    level: Smoother,
    level_g: Glide,
    prev: [bool; 3],
}

impl Looper {
    pub fn new() -> Self {
        Self {
            buf: [Vec::new(), Vec::new()],
            cap: 0,
            len: 0,
            pos: 0,
            state: LoopState::Empty,
            s: LooperSettings::default(),
            level: Smoother::new(1.0),
            level_g: Glide::default(),
            prev: [false; 3],
        }
    }

    /// Allocates 30 s per channel and clears the loop. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        self.cap = (MAX_LOOP_SECONDS * sr) as usize;
        for b in self.buf.iter_mut() {
            *b = vec![0.0; self.cap];
        }
        self.level.set_time(sr, 20.0);
        self.len = 0;
        self.pos = 0;
        self.state = LoopState::Empty;
        self.reset();
    }

    pub fn set(&mut self, s: LooperSettings) {
        self.s = s;
        self.level.set_target(level_gain(s.level_db));
    }

    /// Keeps the loop, stops playback.
    pub fn reset(&mut self) {
        match self.state {
            LoopState::Record => {
                self.finish_recording();
                self.state = if self.len > 0 { LoopState::Stopped } else { LoopState::Empty };
            }
            LoopState::Play | LoopState::Overdub => self.state = LoopState::Stopped,
            _ => {}
        }
        self.pos = self.start_pos();
        let g = level_gain(self.s.level_db);
        self.level.reset(g);
        self.level_g.hold(g);
    }

    pub fn state(&self) -> LoopState {
        self.state
    }

    pub fn length(&self) -> usize {
        self.len
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    fn start_pos(&self) -> usize {
        if self.s.reverse && self.len > 0 {
            self.len - 1
        } else {
            0
        }
    }

    fn finish_recording(&mut self) {
        self.len = self.pos.min(self.cap);
        let f = FADE.min(self.len / 4);
        for i in 0..f {
            let g = i as f32 / f as f32;
            let end = self.len - 1 - i;
            for b in self.buf.iter_mut() {
                b[i] *= g;
                b[end] *= g;
            }
        }
        self.pos = self.start_pos();
    }

    /// Rising-edge detection for the three button parameters.
    pub fn buttons(&mut self, loop_btn: bool, stop: bool, clear: bool) {
        if loop_btn && !self.prev[0] {
            self.press_loop();
        }
        if stop && !self.prev[1] {
            self.press_stop();
        }
        if clear && !self.prev[2] {
            self.press_clear();
        }
        self.prev = [loop_btn, stop, clear];
    }

    pub fn press_loop(&mut self) {
        match self.state {
            LoopState::Empty => {
                if self.cap > 0 {
                    self.state = LoopState::Record;
                    self.pos = 0;
                    self.len = 0;
                }
            }
            LoopState::Record => {
                self.finish_recording();
                self.state = if self.len > 0 { LoopState::Play } else { LoopState::Empty };
            }
            LoopState::Play => self.state = LoopState::Overdub,
            LoopState::Overdub => self.state = LoopState::Play,
            LoopState::Stopped => {
                self.state = LoopState::Play;
                self.pos = self.start_pos();
            }
        }
    }

    pub fn press_stop(&mut self) {
        match self.state {
            LoopState::Record => {
                self.finish_recording();
                self.state = if self.len > 0 { LoopState::Stopped } else { LoopState::Empty };
            }
            LoopState::Play | LoopState::Overdub => {
                self.state = LoopState::Stopped;
                self.pos = self.start_pos();
            }
            _ => {}
        }
    }

    pub fn press_clear(&mut self) {
        self.state = LoopState::Empty;
        self.len = 0;
        self.pos = 0;
    }

    pub fn begin_block(&mut self, frames: usize) {
        let (g0, g1) = self.level.block(frames);
        self.level_g.set(g0, g1, frames);
    }

    fn advance(&mut self) {
        if self.len == 0 {
            return;
        }
        if self.s.reverse {
            self.pos = if self.pos == 0 { self.len - 1 } else { self.pos - 1 };
        } else {
            self.pos += 1;
            if self.pos >= self.len {
                self.pos = 0;
            }
        }
    }

    #[inline]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let g = self.level_g.step();
        match self.state {
            LoopState::Empty | LoopState::Stopped => (l, r),
            LoopState::Record => {
                if self.pos < self.cap {
                    self.buf[0][self.pos] = l;
                    self.buf[1][self.pos] = r;
                    self.pos += 1;
                }
                if self.pos >= self.cap {
                    self.finish_recording();
                    self.state = LoopState::Play;
                }
                (l, r)
            }
            LoopState::Play | LoopState::Overdub => {
                let i = self.pos;
                let a = self.buf[0][i];
                let b = self.buf[1][i];
                if self.state == LoopState::Overdub {
                    self.buf[0][i] = a + l;
                    self.buf[1][i] = b + r;
                }
                self.advance();
                (l + a * g, r + b * g)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;
    use crate::dsp::util::SUB;

    const SR: f32 = 1000.0;

    fn make() -> Looper {
        let mut lp = Looper::new();
        lp.prepare(SR);
        lp
    }

    fn run(lp: &mut Looper, x: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(x.len());
        for chunk in x.chunks(SUB) {
            lp.begin_block(chunk.len());
            for &v in chunk {
                out.push(lp.process_frame(v, v).0);
            }
        }
        out
    }

    fn recorded(lp: &mut Looper, x: &[f32]) {
        lp.press_loop();
        run(lp, x);
        lp.press_loop();
    }

    #[test]
    fn empty_is_exact_passthrough() {
        let mut lp = make();
        let x = noise(500, 0.5, 61);
        assert_eq!(run(&mut lp, &x), x);
    }

    #[test]
    fn record_then_play_repeats_the_loop() {
        let mut lp = make();
        let x = noise(1000, 0.5, 62);
        recorded(&mut lp, &x);
        assert_eq!(lp.state(), LoopState::Play);
        assert_eq!(lp.length(), 1000);
        let y = run(&mut lp, &vec![0.0; 1000]);
        for i in 100..900 {
            assert!((y[i] - x[i]).abs() < 1e-6, "{i}");
        }
    }

    #[test]
    fn overdub_adds_to_the_loop() {
        let mut lp = make();
        let a = noise(1000, 0.3, 63);
        let b = noise(1000, 0.3, 64);
        recorded(&mut lp, &a);
        lp.press_loop();
        assert_eq!(lp.state(), LoopState::Overdub);
        run(&mut lp, &b);
        lp.press_loop();
        assert_eq!(lp.state(), LoopState::Play);
        let y = run(&mut lp, &vec![0.0; 1000]);
        for i in 100..900 {
            assert!((y[i] - (a[i] + b[i])).abs() < 1e-5, "{i}");
        }
    }

    #[test]
    fn stop_then_loop_resumes_from_the_start() {
        let mut lp = make();
        let x = noise(1000, 0.5, 65);
        recorded(&mut lp, &x);
        run(&mut lp, &vec![0.0; 300]);
        lp.press_stop();
        assert_eq!(lp.state(), LoopState::Stopped);
        let silent = run(&mut lp, &vec![0.0; 200]);
        assert!(silent.iter().all(|v| *v == 0.0));
        lp.press_loop();
        assert_eq!(lp.state(), LoopState::Play);
        let y = run(&mut lp, &vec![0.0; 1000]);
        for i in 100..900 {
            assert!((y[i] - x[i]).abs() < 1e-6, "{i}");
        }
    }

    #[test]
    fn clear_empties_the_loop() {
        let mut lp = make();
        recorded(&mut lp, &noise(1000, 0.5, 66));
        lp.press_clear();
        assert_eq!(lp.state(), LoopState::Empty);
        assert_eq!(lp.length(), 0);
        let x = noise(300, 0.5, 67);
        assert_eq!(run(&mut lp, &x), x);
    }

    #[test]
    fn reverse_plays_backwards() {
        let mut lp = make();
        let ramp: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();
        lp.set(LooperSettings { level_db: 0.0, reverse: true });
        recorded(&mut lp, &ramp);
        let y = run(&mut lp, &vec![0.0; 1000]);
        assert!(y[200] > y[300] && y[300] > y[400], "{} {} {}", y[200], y[300], y[400]);
    }

    #[test]
    fn level_minus_60_is_silent() {
        let mut lp = make();
        recorded(&mut lp, &noise(1000, 0.5, 68));
        lp.set(LooperSettings { level_db: -60.0, reverse: false });
        let y = run(&mut lp, &vec![0.0; 1000]);
        assert!(y[200..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn recording_stops_at_30_seconds() {
        let mut lp = make();
        lp.press_loop();
        run(&mut lp, &vec![0.1; 31000]);
        assert_eq!(lp.state(), LoopState::Play);
        assert_eq!(lp.length(), 30000);
    }

    #[test]
    fn buttons_act_on_rising_edges() {
        let mut lp = make();
        lp.buttons(true, false, false);
        assert_eq!(lp.state(), LoopState::Record);
        lp.buttons(true, false, false);
        assert_eq!(lp.state(), LoopState::Record);
        run(&mut lp, &vec![0.1; 100]);
        lp.buttons(false, false, false);
        lp.buttons(true, false, false);
        assert_eq!(lp.state(), LoopState::Play);
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `70 passed`.

**Commit:** `S13: looper module`

---

### S14 — `src/dsp/tuner.rs`

**Design (read only):**
- `Decimator` runs on the audio thread. It averages groups of `round(sr/48000)` samples, so the tuner always sees a rate of about 48 kHz.
- `Yin` runs on the EDITOR thread, only while the window is open, so it costs the audio thread nothing.
- Window 1024, maximum lag 1024, buffer 2048, threshold 0.15. Input quieter than 0.003 RMS gives no reading (`NO SIGNAL`).
- Note names are ASCII (B8).

**Files:**
- Create `src/dsp/tuner.rs` (VERBATIM).
- Edit `src/dsp/mod.rs`: add `pub mod tuner;` below `pub mod looper;`.

`src/dsp/tuner.rs`
```rust
//! Tuner: `Decimator` (audio thread) feeds the shared tap; `Yin` (editor thread) finds the pitch.

pub const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
pub const WINDOW: usize = 1024;
pub const MAX_LAG: usize = 1024;
pub const BUFFER: usize = WINDOW + MAX_LAG;
const THRESHOLD: f32 = 0.15;
const MIN_RMS: f32 = 0.003;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub freq: f32,
    pub note: usize,
    pub octave: i32,
    pub cents: f32,
}

impl Reading {
    pub fn from_freq(freq: f32, reference: f32) -> Self {
        let midi = 69.0 + 12.0 * (freq / reference).log2();
        let n = midi.round();
        let cents = (midi - n) * 100.0;
        let n = n as i32;
        Self { freq, note: n.rem_euclid(12) as usize, octave: n.div_euclid(12) - 1, cents }
    }

    pub fn name(&self) -> &'static str {
        NOTE_NAMES[self.note]
    }
}

/// YIN pitch detector. Allocates in `new`; `detect` does not allocate.
pub struct Yin {
    diff: Vec<f32>,
    cmnd: Vec<f32>,
}

impl Yin {
    pub fn new() -> Self {
        Self { diff: vec![0.0; MAX_LAG + 1], cmnd: vec![0.0; MAX_LAG + 1] }
    }

    /// Uses the newest BUFFER samples of `x` (oldest first). None = silence or no clear pitch.
    pub fn detect(&mut self, x: &[f32], sr: f32) -> Option<f32> {
        if x.len() < BUFFER {
            return None;
        }
        let x = &x[x.len() - BUFFER..];
        let energy = x[..WINDOW].iter().map(|v| v * v).sum::<f32>() / WINDOW as f32;
        if energy.sqrt() < MIN_RMS {
            return None;
        }
        let min_lag = ((sr / 1500.0) as usize).max(2);
        for tau in 1..=MAX_LAG {
            let mut s = 0.0f32;
            for j in 0..WINDOW {
                let d = x[j] - x[j + tau];
                s += d * d;
            }
            self.diff[tau] = s;
        }
        self.cmnd[0] = 1.0;
        let mut running = 0.0f32;
        for tau in 1..=MAX_LAG {
            running += self.diff[tau];
            self.cmnd[tau] = if running > 0.0 { self.diff[tau] * tau as f32 / running } else { 1.0 };
        }
        let mut tau = min_lag;
        let mut found = None;
        while tau < MAX_LAG {
            if self.cmnd[tau] < THRESHOLD {
                while tau + 1 < MAX_LAG && self.cmnd[tau + 1] < self.cmnd[tau] {
                    tau += 1;
                }
                found = Some(tau);
                break;
            }
            tau += 1;
        }
        let tau = found?;
        let (a, b, c) = (self.cmnd[tau - 1], self.cmnd[tau], self.cmnd[tau + 1]);
        let denom = a - 2.0 * b + c;
        let shift = if denom.abs() > 1e-12 { 0.5 * (a - c) / denom } else { 0.0 };
        Some(sr / (tau as f32 + shift.clamp(-1.0, 1.0)))
    }
}

/// Averages groups of samples so the tuner always sees about 48 kHz.
pub struct Decimator {
    factor: usize,
    acc: f32,
    count: usize,
}

impl Decimator {
    pub fn new() -> Self {
        Self { factor: 1, acc: 0.0, count: 0 }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.factor = ((sr / 48000.0).round() as usize).max(1);
        self.acc = 0.0;
        self.count = 0;
    }

    pub fn factor(&self) -> usize {
        self.factor
    }

    #[inline]
    pub fn push(&mut self, x: f32) -> Option<f32> {
        self.acc += x;
        self.count += 1;
        if self.count == self.factor {
            let y = self.acc / self.factor as f32;
            self.acc = 0.0;
            self.count = 0;
            Some(y)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::testutil::*;

    const SR: f32 = 48000.0;

    fn detect(x: &[f32]) -> Option<f32> {
        Yin::new().detect(x, SR)
    }

    #[test]
    fn finds_low_e() {
        let f = detect(&sine(82.41, SR, BUFFER, 0.3)).expect("pitch");
        let r = Reading::from_freq(f, 440.0);
        assert_eq!(r.name(), "E");
        assert_eq!(r.octave, 2);
        assert!(r.cents.abs() < 2.0, "{}", r.cents);
    }

    #[test]
    fn finds_a4() {
        let f = detect(&sine(440.0, SR, BUFFER, 0.3)).expect("pitch");
        let r = Reading::from_freq(f, 440.0);
        assert_eq!(r.name(), "A");
        assert_eq!(r.octave, 4);
        assert!(r.cents.abs() < 2.0, "{}", r.cents);
    }

    #[test]
    fn cents_follow_the_reference() {
        let f = detect(&sine(445.0, SR, BUFFER, 0.3)).expect("pitch");
        let r = Reading::from_freq(f, 440.0);
        assert!((r.cents - 19.56).abs() < 2.0, "{}", r.cents);
        let r432 = Reading::from_freq(432.0, 432.0);
        assert_eq!(r432.name(), "A");
        assert!(r432.cents.abs() < 0.01);
    }

    #[test]
    fn harmonics_do_not_cause_octave_errors() {
        let mut x = vec![0.0f32; BUFFER];
        for k in 1..=8 {
            let h = sine(110.0 * k as f32, SR, BUFFER, 0.3 / k as f32);
            for i in 0..BUFFER {
                x[i] += h[i];
            }
        }
        let r = Reading::from_freq(detect(&x).expect("pitch"), 440.0);
        assert_eq!(r.name(), "A");
        assert_eq!(r.octave, 2);
    }

    #[test]
    fn silence_gives_no_reading() {
        assert!(detect(&noise(BUFFER, 0.001, 71)).is_none());
        assert!(detect(&vec![0.0; 100]).is_none());
    }

    #[test]
    fn decimator_matches_rates() {
        let mut d = Decimator::new();
        d.set_sample_rate(96000.0);
        assert_eq!(d.factor(), 2);
        assert_eq!(d.push(1.0), None);
        assert_eq!(d.push(3.0), Some(2.0));
        d.set_sample_rate(44100.0);
        assert_eq!(d.factor(), 1);
        d.set_sample_rate(192000.0);
        assert_eq!(d.factor(), 4);
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `76 passed`.

**Commit:** `S14: tuner (YIN + decimator)`

---

### S15 — `src/shared.rs` and `src/chain.rs`

**Design (read only):**

| Item | Rule |
|---|---|
| `Shared` | Atomics only. Holds the tuner tap (4096 samples), the sample rate, and the looper state, length and position. |
| Chain order | Tuner tap (dry, before Input) → Input gain → [oversampled: Drive → Fuzz → Ring] → Vibe → Prism → Echo → Looper → Output gain → Mute. |
| Sub-blocks | 32 samples (B4). |
| Oversampling | The oversampler always runs at the Quality factor, even when the dirt pedals are off. This keeps latency at 24 samples. |
| Quality change | Changing Quality re-rates and resets Drive, Fuzz and Ring. |
| Settings | `ChainSettings` is plain data. `lib.rs` builds it from the parameters once per host block. |

**Files:**
- Create `src/shared.rs` and `src/chain.rs` (VERBATIM).
- Edit `src/lib.rs`: add `pub mod chain;` and `pub mod shared;` directly below `pub mod params;`.

`src/shared.rs`
```rust
//! State shared between the audio thread and the editor. Lock-free: atomics only.

use crate::dsp::looper::LoopState;
use std::sync::atomic::{AtomicU32, AtomicU8, AtomicUsize, Ordering};

pub const TAP_SIZE: usize = 4096;

/// An f32 stored in an AtomicU32.
pub struct SharedF32(AtomicU32);

impl SharedF32 {
    pub fn new(v: f32) -> Self {
        Self(AtomicU32::new(v.to_bits()))
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn store(&self, v: f32) {
        self.0.store(v.to_bits(), Ordering::Relaxed)
    }
}

/// Ring of the most recent decimated dry input samples, for the tuner display.
pub struct TunerTap {
    samples: Vec<AtomicU32>,
    write: AtomicUsize,
    rate: SharedF32,
}

impl Default for TunerTap {
    fn default() -> Self {
        Self::new()
    }
}

impl TunerTap {
    pub fn new() -> Self {
        Self {
            samples: (0..TAP_SIZE).map(|_| AtomicU32::new(0)).collect(),
            write: AtomicUsize::new(0),
            rate: SharedF32::new(48000.0),
        }
    }

    /// Audio thread.
    #[inline]
    pub fn push(&self, x: f32) {
        let w = self.write.load(Ordering::Relaxed);
        self.samples[w % TAP_SIZE].store(x.to_bits(), Ordering::Relaxed);
        self.write.store(w.wrapping_add(1), Ordering::Release);
    }

    pub fn set_rate(&self, sr: f32) {
        self.rate.store(sr);
    }

    pub fn rate(&self) -> f32 {
        self.rate.load()
    }

    /// Total samples written so far (wraps).
    pub fn written(&self) -> usize {
        self.write.load(Ordering::Acquire)
    }

    /// Editor thread: copies the newest `out.len()` samples, oldest first.
    pub fn copy_latest(&self, out: &mut [f32]) {
        let w = self.write.load(Ordering::Acquire);
        let n = out.len().min(TAP_SIZE);
        for i in 0..n {
            let idx = w.wrapping_sub(n).wrapping_add(i) % TAP_SIZE;
            out[i] = f32::from_bits(self.samples[idx].load(Ordering::Relaxed));
        }
    }
}

pub struct Shared {
    pub tuner: TunerTap,
    pub sample_rate: SharedF32,
    loop_state: AtomicU8,
    loop_len: AtomicUsize,
    loop_pos: AtomicUsize,
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

impl Shared {
    pub fn new() -> Self {
        Self {
            tuner: TunerTap::new(),
            sample_rate: SharedF32::new(48000.0),
            loop_state: AtomicU8::new(0),
            loop_len: AtomicUsize::new(0),
            loop_pos: AtomicUsize::new(0),
        }
    }

    pub fn publish_looper(&self, state: LoopState, len: usize, pos: usize) {
        self.loop_state.store(state.to_u8(), Ordering::Relaxed);
        self.loop_len.store(len, Ordering::Relaxed);
        self.loop_pos.store(pos, Ordering::Relaxed);
    }

    /// (state, length in samples, position in samples).
    pub fn looper(&self) -> (LoopState, usize, usize) {
        (
            LoopState::from_u8(self.loop_state.load(Ordering::Relaxed)),
            self.loop_len.load(Ordering::Relaxed),
            self.loop_pos.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuner_tap_returns_newest_samples_oldest_first() {
        let tap = TunerTap::new();
        for i in 0..5000 {
            tap.push(i as f32);
        }
        let mut out = [0.0f32; 4];
        tap.copy_latest(&mut out);
        assert_eq!(out, [4996.0, 4997.0, 4998.0, 4999.0]);
    }
}
```

`src/chain.rs`
```rust
//! The fixed signal chain. Plain Rust: no nih-plug. Built from `ChainSettings` once per host block.

use crate::dsp::drive::{Drive, DriveSettings};
use crate::dsp::echo::{Echo, EchoSettings};
use crate::dsp::fuzz::{Fuzz, FuzzSettings};
use crate::dsp::looper::{Looper, LooperSettings};
use crate::dsp::oversample::{Oversampler, OS_LATENCY};
use crate::dsp::prism::{Prism, PrismSettings};
use crate::dsp::ring::{Ring, RingSettings};
use crate::dsp::tuner::Decimator;
use crate::dsp::util::{db_to_gain, Glide, Smoother, Switch, SUB};
use crate::dsp::vibe::{Vibe, VibeSettings};
use crate::shared::Shared;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainSettings {
    pub input_db: f32,
    pub output_db: f32,
    /// Oversampling factor: 1, 2 or 4.
    pub quality: usize,
    pub tuner_mute: bool,
    pub drive: DriveSettings,
    pub fuzz: FuzzSettings,
    pub ring: RingSettings,
    pub vibe: VibeSettings,
    pub prism: PrismSettings,
    pub echo: EchoSettings,
    pub looper: LooperSettings,
    pub loop_btn: bool,
    pub stop_btn: bool,
    pub clear_btn: bool,
}

impl Default for ChainSettings {
    fn default() -> Self {
        Self {
            input_db: 0.0,
            output_db: 0.0,
            quality: 4,
            tuner_mute: false,
            drive: DriveSettings::default(),
            fuzz: FuzzSettings::default(),
            ring: RingSettings::default(),
            vibe: VibeSettings::default(),
            prism: PrismSettings::default(),
            echo: EchoSettings::default(),
            looper: LooperSettings::default(),
            loop_btn: false,
            stop_btn: false,
            clear_btn: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopButton {
    Loop,
    Stop,
    Clear,
}

pub struct Chain {
    sr: f32,
    factor: usize,
    muted: bool,
    os: Oversampler,
    drive: Drive,
    fuzz: Fuzz,
    ring: Ring,
    vibe: Vibe,
    prism: Prism,
    echo: Echo,
    looper: Looper,
    decim: Decimator,
    input: Smoother,
    output: Smoother,
    in_g: Glide,
    out_g: Glide,
    unmute: Switch,
    mute_g: Glide,
}

impl Chain {
    /// Small allocations only. Call `prepare` before processing.
    pub fn new() -> Self {
        Self {
            sr: 48000.0,
            factor: 4,
            muted: false,
            os: Oversampler::new(),
            drive: Drive::new(),
            fuzz: Fuzz::new(),
            ring: Ring::new(),
            vibe: Vibe::new(),
            prism: Prism::new(),
            echo: Echo::new(),
            looper: Looper::new(),
            decim: Decimator::new(),
            input: Smoother::new(0.0),
            output: Smoother::new(0.0),
            in_g: Glide::default(),
            out_g: Glide::default(),
            unmute: Switch::new(true),
            mute_g: Glide::default(),
        }
    }

    /// Allocates all delay lines and the looper. NOT for the audio thread.
    pub fn prepare(&mut self, sr: f32) {
        self.sr = sr;
        self.prism.prepare(sr);
        self.echo.prepare(sr);
        self.looper.prepare(sr);
        self.vibe.set_sample_rate(sr);
        self.set_os_rate(self.factor);
        self.decim.set_sample_rate(sr);
        self.input.set_time(sr, 20.0);
        self.output.set_time(sr, 20.0);
        self.unmute.set_sample_rate(sr);
        self.reset();
    }

    pub fn latency(&self) -> usize {
        OS_LATENCY
    }

    pub fn tuner_rate(&self) -> f32 {
        self.sr / self.decim.factor() as f32
    }

    fn set_os_rate(&mut self, factor: usize) {
        self.factor = factor;
        let osr = self.sr * factor as f32;
        self.drive.set_sample_rate(osr);
        self.fuzz.set_sample_rate(osr);
        self.ring.set_sample_rate(osr);
    }

    /// Sets all targets (no button handling). Safe on the audio thread.
    pub fn apply(&mut self, s: &ChainSettings) {
        let factor = match s.quality {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        if factor != self.factor {
            self.set_os_rate(factor);
            self.drive.reset();
            self.fuzz.reset();
            self.ring.reset();
        }
        self.drive.set(s.drive);
        self.fuzz.set(s.fuzz);
        self.ring.set(s.ring);
        self.vibe.set(s.vibe);
        self.prism.set(s.prism);
        self.echo.set(s.echo);
        self.looper.set(s.looper);
        self.input.set_target(s.input_db);
        self.output.set_target(s.output_db);
        self.muted = s.tuner_mute;
        self.unmute.set(!s.tuner_mute);
    }

    /// Clears audio state and snaps every smoother to its target.
    pub fn reset(&mut self) {
        self.os.reset();
        self.drive.reset();
        self.fuzz.reset();
        self.ring.reset();
        self.vibe.reset();
        self.prism.reset();
        self.echo.reset();
        self.looper.reset();
        self.decim.set_sample_rate(self.sr);
        let (i, o) = (self.input.target(), self.output.target());
        self.input.reset(i);
        self.output.reset(o);
        self.in_g.hold(db_to_gain(i));
        self.out_g.hold(db_to_gain(o));
        self.unmute.reset(!self.muted);
        self.mute_g.hold(self.unmute.value());
    }

    /// MIDI foot-controller presses (plan.md B9).
    pub fn press(&mut self, b: LoopButton) {
        match b {
            LoopButton::Loop => self.looper.press_loop(),
            LoopButton::Stop => self.looper.press_stop(),
            LoopButton::Clear => self.looper.press_clear(),
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], s: &ChainSettings, shared: &Shared) {
        self.looper.buttons(s.loop_btn, s.stop_btn, s.clear_btn);
        self.apply(s);
        let n = left.len().min(right.len());
        let mut start = 0;
        while start < n {
            let len = SUB.min(n - start);
            self.process_sub(&mut left[start..start + len], &mut right[start..start + len], shared);
            start += len;
        }
        shared.publish_looper(self.looper.state(), self.looper.length(), self.looper.position());
    }

    fn process_sub(&mut self, l: &mut [f32], r: &mut [f32], shared: &Shared) {
        let n = l.len();
        // 1. Tuner tap: dry input, before anything touches it.
        for i in 0..n {
            if let Some(v) = self.decim.push(0.5 * (l[i] + r[i])) {
                shared.tuner.push(v);
            }
        }
        // 2. Input gain.
        let (a, b) = self.input.block(n);
        self.in_g.set(db_to_gain(a), db_to_gain(b), n);
        for i in 0..n {
            let g = self.in_g.step();
            l[i] *= g;
            r[i] *= g;
        }
        // 3. Oversampled dirt section.
        let factor = self.factor;
        let frames = n * factor;
        self.drive.begin_block(frames);
        self.fuzz.begin_block(frames);
        self.ring.begin_block(frames);
        let (os, drive, fuzz, ring) = (&mut self.os, &mut self.drive, &mut self.fuzz, &mut self.ring);
        os.process(factor, l, r, |a, b| {
            let (a, b) = drive.process_frame(a, b);
            let (a, b) = fuzz.process_frame(a, b);
            ring.process_frame(a, b)
        });
        // 4. Host-rate section.
        self.vibe.begin_block(n);
        self.prism.begin_block(n);
        self.echo.begin_block(n);
        self.looper.begin_block(n);
        let (o0, o1) = self.output.block(n);
        self.out_g.set(db_to_gain(o0), db_to_gain(o1), n);
        let (m0, m1) = self.unmute.block(n);
        self.mute_g.set(m0, m1, n);
        for i in 0..n {
            let (a, b) = self.vibe.process_frame(l[i], r[i]);
            let (a, b) = self.prism.process_frame(a, b);
            let (a, b) = self.echo.process_frame(a, b);
            let (a, b) = self.looper.process_frame(a, b);
            let g = self.out_g.step() * self.mute_g.step();
            l[i] = a * g;
            r[i] = b * g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::looper::LoopState;
    use crate::dsp::testutil::*;
    use crate::dsp::tuner::{Yin, BUFFER};

    const SR: f32 = 48000.0;

    fn all_off(quality: usize) -> ChainSettings {
        let mut s = ChainSettings { quality, ..Default::default() };
        s.drive.on = false;
        s.fuzz.on = false;
        s.ring.on = false;
        s.vibe.on = false;
        s.prism.on = false;
        s.echo.on = false;
        s.echo.trails = false;
        s
    }

    fn make(s: &ChainSettings) -> (Chain, Shared) {
        let mut c = Chain::new();
        c.prepare(SR);
        c.apply(s);
        c.reset();
        (c, Shared::new())
    }

    fn run(c: &mut Chain, sh: &Shared, s: &ChainSettings, x: &[f32]) -> Vec<f32> {
        let mut l = x.to_vec();
        let mut r = x.to_vec();
        for (a, b) in l.chunks_mut(512).zip(r.chunks_mut(512)) {
            c.process(a, b, s, sh);
        }
        l
    }

    #[test]
    fn latency_is_24_samples_at_every_quality() {
        for q in [1usize, 2, 4] {
            let s = all_off(q);
            let (mut c, sh) = make(&s);
            let mut x = vec![0.0f32; 512];
            x[0] = 1.0;
            let y = run(&mut c, &sh, &s, &x);
            assert_eq!(c.latency(), 24);
            assert_eq!(argmax_abs(&y), 24, "quality {q}");
        }
    }

    #[test]
    fn default_board_is_finite_and_bounded() {
        let s = ChainSettings::default();
        let (mut c, sh) = make(&s);
        let y = run(&mut c, &sh, &s, &noise(48000, 0.3, 81));
        assert!(all_finite(&y));
        assert!(peak(&y) < 8.0, "{}", peak(&y));
    }

    #[test]
    fn tuner_mute_silences_output_but_tuner_still_listens() {
        let s = ChainSettings { tuner_mute: true, ..all_off(4) };
        let (mut c, sh) = make(&s);
        let y = run(&mut c, &sh, &s, &sine(440.0, SR, 9600, 0.3));
        assert!(y[1000..].iter().all(|v| *v == 0.0));
        let mut buf = vec![0.0f32; BUFFER];
        sh.tuner.copy_latest(&mut buf);
        let f = Yin::new().detect(&buf, sh.tuner.rate()).expect("pitch");
        assert!((f - 440.0).abs() < 1.0, "{f}");
    }

    #[test]
    fn loop_button_is_published() {
        let s = ChainSettings { loop_btn: true, ..all_off(1) };
        let (mut c, sh) = make(&s);
        run(&mut c, &sh, &s, &vec![0.1; 512]);
        assert_eq!(sh.looper().0, LoopState::Record);
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `81 passed`.

**Commit:** `S15: shared state and signal chain`

---

### S16 — Final `lib.rs` wiring, bundle, pluginval, HUMAN CHECK 1

**Goal:** a real, audible plugin. The host's generic parameter UI is enough to use it at this point.

**1. Replace `src/lib.rs` completely** (VERBATIM):

```rust
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::new_without_default)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]

pub mod chain;
pub mod dsp;
pub mod fmt;
pub mod params;
pub mod shared;

use chain::{Chain, ChainSettings, LoopButton};
use dsp::drive::DriveSettings;
use dsp::echo::EchoSettings;
use dsp::fuzz::FuzzSettings;
use dsp::looper::LooperSettings;
use dsp::prism::PrismSettings;
use dsp::ring::RingSettings;
use dsp::vibe::VibeSettings;
use nih_plug::prelude::*;
use params::HolyMolyParams;
use shared::Shared;
use std::sync::Arc;

pub struct HolyMoly {
    params: Arc<HolyMolyParams>,
    shared: Arc<Shared>,
    chain: Chain,
    mono_in: bool,
}

impl Default for HolyMoly {
    fn default() -> Self {
        Self {
            params: Arc::new(HolyMolyParams::default()),
            shared: Arc::new(Shared::new()),
            chain: Chain::new(),
            mono_in: false,
        }
    }
}

impl HolyMoly {
    /// Reads every parameter once per host block (plan.md B3).
    fn settings(&self, tempo: Option<f64>) -> ChainSettings {
        let p = &self.params;
        ChainSettings {
            input_db: p.input.value(),
            output_db: p.output.value(),
            quality: p.quality.value().factor(),
            tuner_mute: p.tuner.mute.value(),
            drive: DriveSettings {
                on: p.drive.on.value(),
                dist: p.drive.dist.value(),
                tone: p.drive.tone.value(),
                level_db: p.drive.level.value(),
                voice: p.drive.voice.value(),
            },
            fuzz: FuzzSettings {
                on: p.fuzz.on.value(),
                fuzz: p.fuzz.fuzz.value(),
                bias: p.fuzz.bias.value(),
                tone: p.fuzz.tone.value(),
                volume_db: p.fuzz.volume.value(),
            },
            ring: RingSettings {
                on: p.ring.on.value(),
                gain_db: p.ring.gain.value(),
                dist: p.ring.dist.value(),
                modulation: p.ring.modulation.value(),
                output_db: p.ring.output.value(),
                mode: p.ring.mode.value(),
                freq_hz: p.ring.freq.value(),
            },
            vibe: VibeSettings {
                on: p.vibe.on.value(),
                rate_hz: p.vibe.rate.value(),
                depth: p.vibe.depth.value(),
                feedback: p.vibe.feedback.value(),
                mix: p.vibe.mix.value(),
                mode: p.vibe.mode.value(),
            },
            prism: PrismSettings {
                on: p.prism.on.value(),
                pitch_st: p.prism.pitch.value(),
                primary: p.prism.primary.value(),
                secondary: p.prism.secondary.value(),
                tracking_ms: p.prism.tracking.value(),
                depth: p.prism.depth.value(),
                halo: p.prism.halo.value(),
                tone_hz: p.prism.tone.value(),
            },
            echo: EchoSettings {
                on: p.echo.on.value(),
                time_ms: p.echo.time.value(),
                sync: p.echo.sync.value(),
                division: p.echo.division.value(),
                feedback: p.echo.feedback.value(),
                mix: p.echo.mix.value(),
                tone_hz: p.echo.tone.value(),
                modulation: p.echo.modulation.value(),
                mode: p.echo.mode.value(),
                pingpong: p.echo.pingpong.value(),
                trails: p.echo.trails.value(),
                tempo_bpm: tempo.unwrap_or(120.0) as f32,
            },
            looper: LooperSettings { level_db: p.looper.level.value(), reverse: p.looper.reverse.value() },
            loop_btn: p.looper.loop_btn.value(),
            stop_btn: p.looper.stop_btn.value(),
            clear_btn: p.looper.clear_btn.value(),
        }
    }
}

impl Plugin for HolyMoly {
    const NAME: &'static str = "Holy Moly";
    const VENDOR: &'static str = "Holy Moly Audio";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.mono_in = audio_io_layout.main_input_channels.map(|c| c.get()) == Some(1);
        let sr = buffer_config.sample_rate;
        self.chain.prepare(sr);
        self.shared.sample_rate.store(sr);
        self.shared.tuner.set_rate(self.chain.tuner_rate());
        context.set_latency_samples(self.chain.latency() as u32);
        true
    }

    fn reset(&mut self) {
        self.chain.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            if let NoteEvent::NoteOn { note, .. } = event {
                match note {
                    60 => self.chain.press(LoopButton::Loop),
                    61 => self.chain.press(LoopButton::Stop),
                    62 => self.chain.press(LoopButton::Clear),
                    _ => {}
                }
            }
        }
        let s = self.settings(context.transport().tempo);
        let channels = buffer.as_slice();
        if channels.len() < 2 {
            return ProcessStatus::Normal;
        }
        let (first, rest) = channels.split_at_mut(1);
        let left = &mut *first[0];
        let right = &mut *rest[0];
        if self.mono_in {
            right.copy_from_slice(left);
        }
        self.chain.process(left, right, &s, &self.shared);
        ProcessStatus::KeepAlive
    }
}

impl Vst3Plugin for HolyMoly {
    const VST3_CLASS_ID: [u8; 16] = *b"HolyMolyBoard001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Delay,
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_vst3!(HolyMoly);
```

**2. Tighten the placeholder scan.** Edit `scripts/check.ps1`: change the pattern to `'todo!\(|unimplemented!\(|TODO|FIXME|SCAFFOLD'`. From now on, `SCAFFOLD` is forbidden everywhere.

**3. Edit `.gitignore`:** add the line `/tools`.

**4. VERIFY the build.**
- CHECK → `CHECK PASSED`, with at least `82 passed`.
- BUNDLE → `True`.

**5. Run pluginval** (level 5, no GUI yet):

```powershell
New-Item -ItemType Directory -Force tools | Out-Null
if (-not (Test-Path tools\pluginval.zip)) {
  Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/Tracktion/pluginval/releases/latest/download/pluginval_Windows.zip" -OutFile tools\pluginval.zip
}
Expand-Archive -Force tools\pluginval.zip tools\pluginval
$pv = Get-ChildItem -Recurse tools\pluginval -Filter pluginval.exe | Select-Object -First 1
$p = Start-Process -FilePath $pv.FullName -ArgumentList '--strictness-level 5 --skip-gui-tests --validate "target\bundled\Holy Moly.vst3"' -Wait -PassThru -NoNewWindow -RedirectStandardOutput target\pluginval.log
"pluginval exit code: $($p.ExitCode)"
Get-Content target\pluginval.log -Tail 15
```

EXPECTED: `pluginval exit code: 0`.

If the download fails, add HUMAN CHECK `Run pluginval on target\bundled\Holy Moly.vst3` and continue. If pluginval reports failures, see Part E, E-V1.

**6. Add HUMAN CHECK 1** to `HUMAN_CHECKS.md`:

`HC1 (S16): Copy target\bundled\Holy Moly.vst3 to C:\Program Files\Common Files\VST3\. In Reaper, put Holy Moly on a guitar or drum track and use the generic parameter UI. Check: sound passes; the drive and fuzz footswitch parameters audibly change the tone; Echo repeats at 400 ms; turning Echo off with Trails on lets the repeats ring out; project save and reload keeps the settings.`

Continue immediately; do not wait for the result.

**Commit:** `S16: complete audio plugin, pluginval level 5`

---

### S17 — Fonts

**Goal:** embed the five font files (C3). If the download fails, the editor falls back to egui's own fonts (B14), and the build never breaks.

**1. Create `scripts/get_fonts.ps1`** (VERBATIM):

```powershell
# Downloads static font instances from the Google Fonts CSS API. Prints FONTS OK or FONTS INCOMPLETE.
$fonts = @(
  @{ File = "BigShouldersStencil-800.ttf"; Url = "https://fonts.googleapis.com/css2?family=Big+Shoulders+Stencil:wght@800" },
  @{ File = "BigShouldersStencil-900.ttf"; Url = "https://fonts.googleapis.com/css2?family=Big+Shoulders+Stencil:wght@900" },
  @{ File = "MartianMono-Condensed-500.ttf"; Url = "https://fonts.googleapis.com/css2?family=Martian+Mono:wdth,wght@75,500" },
  @{ File = "MartianMono-Condensed-600.ttf"; Url = "https://fonts.googleapis.com/css2?family=Martian+Mono:wdth,wght@75,600" },
  @{ File = "MartianMono-Condensed-700.ttf"; Url = "https://fonts.googleapis.com/css2?family=Martian+Mono:wdth,wght@75,700" }
)
$ok = $true
foreach ($f in $fonts) {
  $dest = Join-Path "assets\fonts" $f.File
  try {
    $css = (Invoke-WebRequest -UseBasicParsing -Uri $f.Url -UserAgent "Wget/1.21").Content
    $m = [regex]::Match($css, 'url\((https://[^)]+)\)')
    if (-not $m.Success) { throw "no font url in css" }
    Invoke-WebRequest -UseBasicParsing -Uri $m.Groups[1].Value -OutFile $dest -UserAgent "Wget/1.21"
    $bytes = [System.IO.File]::ReadAllBytes($dest)
    $magic = [System.BitConverter]::ToString($bytes, 0, 4)
    if ($bytes.Length -lt 20000 -or ($magic -ne "00-01-00-00" -and $magic -ne "4F-54-54-4F")) { throw "not a TTF/OTF ($magic, $($bytes.Length) bytes)" }
    Write-Host "OK   $($f.File)  $($bytes.Length) bytes"
  } catch {
    Write-Host "FAIL $($f.File): $_"
    if (Test-Path $dest) { Remove-Item $dest }
    $ok = $false
  }
}
foreach ($lic in @(@{ Dir = "martianmono"; Out = "OFL-MartianMono.txt" }, @{ Dir = "bigshouldersstencil"; Out = "OFL-BigShouldersStencil.txt" })) {
  try {
    Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/google/fonts/main/ofl/$($lic.Dir)/OFL.txt" -OutFile (Join-Path "assets\fonts" $lic.Out)
    Write-Host "OK   $($lic.Out)"
  } catch { Write-Host "FAIL licence $($lic.Out)" }
}
if ($ok) { Write-Host "FONTS OK" } else { Write-Host "FONTS INCOMPLETE" }
```

**2. Run it:**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\get_fonts.ps1
```

EXPECTED: `FONTS OK`.

If the result is `FONTS INCOMPLETE`:
1. Add HUMAN CHECK `HC-F: Put the missing static TTFs into assets\fonts (names in plan.md C3), then rebuild.`
2. Note the missing files in PROGRESS.md.
3. Continue. The code below still compiles.

If a licence download failed, add HUMAN CHECK `HC-L: add the OFL licence texts for both fonts to assets\fonts before release.`

**3. Create `build.rs`** in the project root, next to `Cargo.toml` (VERBATIM):

```rust
// Enables cfg(hm_fonts) only when all five font files exist (plan.md B14).
fn main() {
    println!("cargo::rustc-check-cfg=cfg(hm_fonts)");
    let files = [
        "BigShouldersStencil-800.ttf",
        "BigShouldersStencil-900.ttf",
        "MartianMono-Condensed-500.ttf",
        "MartianMono-Condensed-600.ttf",
        "MartianMono-Condensed-700.ttf",
    ];
    let dir = std::path::Path::new("assets").join("fonts");
    let mut all = true;
    for f in files {
        println!("cargo::rerun-if-changed=assets/fonts/{f}");
        if !dir.join(f).exists() {
            all = false;
        }
    }
    if all {
        println!("cargo::rustc-cfg=hm_fonts");
    }
}
```

**4. Create `src/editor/mod.rs`** (temporary; replaced in S18):

```rust
//! Editor. Grows in Steps S18 to S25.
pub mod fonts;
```

**5. Create `src/editor/fonts.rs`** (VERBATIM):

```rust
//! Font registration (plan.md C3, B14). Families: display800, display900, mono500, mono600, mono700.
//! Each family falls back to egui's default fonts for missing glyphs.

use nih_plug_egui::egui::{self, FontData, FontDefinitions, FontFamily};

pub const FAMILIES: [&str; 5] = ["display800", "display900", "mono500", "mono600", "mono700"];

#[cfg(hm_fonts)]
const FILES: [(&str, &[u8]); 5] = [
    ("display800", include_bytes!("../../assets/fonts/BigShouldersStencil-800.ttf")),
    ("display900", include_bytes!("../../assets/fonts/BigShouldersStencil-900.ttf")),
    ("mono500", include_bytes!("../../assets/fonts/MartianMono-Condensed-500.ttf")),
    ("mono600", include_bytes!("../../assets/fonts/MartianMono-Condensed-600.ttf")),
    ("mono700", include_bytes!("../../assets/fonts/MartianMono-Condensed-700.ttf")),
];

#[cfg(not(hm_fonts))]
const FILES: [(&str, &[u8]); 0] = [];

/// True when the real fonts were embedded at build time.
pub fn embedded() -> bool {
    cfg!(hm_fonts)
}

pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mono_fallback = fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let prop_fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (name, bytes) in FILES.iter() {
        fonts.font_data.insert((*name).to_owned(), FontData::from_static(bytes).into());
    }
    for name in FAMILIES {
        let mut list = Vec::new();
        if FILES.iter().any(|(n, _)| *n == name) {
            list.push(name.to_owned());
        }
        let fallback = if name.starts_with("mono") { &mono_fallback } else { &prop_fallback };
        list.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), list);
    }
    fonts
}

pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_is_registered() {
        let defs = definitions();
        for name in FAMILIES {
            assert!(defs.families.contains_key(&FontFamily::Name(name.into())), "{name}");
        }
    }
}
```

**6. Edit `src/lib.rs`:** add `pub mod editor;` below `pub mod dsp;`.

**7. Edit `.gitignore`:** fonts ARE committed, so add nothing.

**VERIFY:** CHECK → `CHECK PASSED`, with at least `83 passed`. Then:

```powershell
cargo build 2>&1 | Select-String "hm_fonts"
```

No output is expected. Record `fonts embedded: yes/no` in PROGRESS.md (yes when `FONTS OK`).

**Commit:** `S17: fonts with fallback`

---

### S18 — Editor skeleton, standalone build, screenshot tools

**Goal:** the plugin opens a 1120 × 760 window showing the board background. The agent also gets "eyes": a standalone build plus two screenshot scripts.

**1. Edit `Cargo.toml`:** add `"standalone"` to the nih_plug features:

```toml
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "<HASH>", features = ["assert_process_allocs", "standalone"] }
```

**2. Create `src/main.rs`** (VERBATIM):

```rust
// Standalone build for visual checks: target\release\holy_moly.exe --backend dummy
use holy_moly::HolyMoly;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<HolyMoly>();
}
```

**3. Create `src/editor/theme.rs`** (VERBATIM):

```rust
//! Colour tokens (plan.md C2) and the pedal table (C4.2).

use nih_plug_egui::egui::Color32;

pub const EDITOR_W: f32 = 1120.0;
pub const EDITOR_H: f32 = 760.0;
pub const PEDAL_H: f32 = 300.0;

pub const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub const GROUND: Color32 = hex(0x0a0c07);
pub const DECK: Color32 = hex(0x12150d);
pub const PANEL: Color32 = hex(0x1b1f14);
pub const WELL: Color32 = hex(0x050604);
pub const LINE: Color32 = hex(0x2b3121);
pub const LINE_STRONG: Color32 = hex(0x616d47);
pub const INK: Color32 = hex(0xeef3d6);
pub const INK_MUTED: Color32 = hex(0xa8b18b);
pub const INK_FAINT: Color32 = hex(0x687150);
pub const ACID: Color32 = hex(0xe4ff1a);
pub const ACID_DIM: Color32 = hex(0x5e6a0a);
pub const ACID_DEEP: Color32 = hex(0x252b06);
pub const ON_ACID: Color32 = hex(0x0a0c07);
pub const DECAY: Color32 = hex(0xff6b1a);
pub const DECAY_DEEP: Color32 = hex(0x3a1606);
pub const ON_DECAY: Color32 = hex(0x0a0c07);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Finish {
    pub body: Color32,
    pub print: Color32,
}

pub const FINISH_ACID: Finish = Finish { body: hex(0xe4ff1a), print: hex(0x0a0c07) };
pub const FINISH_BLACK: Finish = Finish { body: hex(0x161912), print: hex(0xe4ff1a) };
pub const FINISH_DECAY: Finish = Finish { body: hex(0xff6b1a), print: hex(0x0a0c07) };
pub const FINISH_BONE: Finish = Finish { body: hex(0xe4e8cc), print: hex(0x1d5a2a) };
pub const FINISH_CHERENKOV: Finish = Finish { body: hex(0x0a2431), print: hex(0x47d9ff) };
pub const FINISH_OLIVE: Finish = Finish { body: hex(0x394020), print: hex(0xe4ff1a) };
pub const FINISH_LEAD: Finish = Finish { body: hex(0x262b1e), print: hex(0xe4ff1a) };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Art {
    Stripes,
    Halftone,
    Rings,
    Diamonds,
    Fan,
    Echoes,
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct PedalGeom {
    pub name: &'static str,
    pub stage: &'static str,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub finish: Finish,
    pub art: Art,
}

pub const DRIVE: PedalGeom = PedalGeom { name: "Drive", stage: "02", x: 16.0, y: 136.0, w: 240.0, finish: FINISH_ACID, art: Art::Stripes };
pub const FUZZ: PedalGeom = PedalGeom { name: "Fuzz", stage: "03", x: 268.0, y: 136.0, w: 232.0, finish: FINISH_BLACK, art: Art::Halftone };
pub const RING: PedalGeom = PedalGeom { name: "Ring", stage: "04", x: 512.0, y: 136.0, w: 316.0, finish: FINISH_DECAY, art: Art::Rings };
pub const VIBE: PedalGeom = PedalGeom { name: "Vibe", stage: "05", x: 840.0, y: 136.0, w: 264.0, finish: FINISH_BONE, art: Art::Diamonds };
pub const PRISM: PedalGeom = PedalGeom { name: "Prism", stage: "06", x: 16.0, y: 448.0, w: 336.0, finish: FINISH_CHERENKOV, art: Art::Fan };
pub const ECHO: PedalGeom = PedalGeom { name: "Echo", stage: "07", x: 364.0, y: 448.0, w: 476.0, finish: FINISH_OLIVE, art: Art::Echoes };
pub const LOOPER: PedalGeom = PedalGeom { name: "Looper", stage: "08", x: 852.0, y: 448.0, w: 252.0, finish: FINISH_LEAD, art: Art::Orbit };

pub const PEDALS: [PedalGeom; 7] = [DRIVE, FUZZ, RING, VIBE, PRISM, ECHO, LOOPER];
```

**4. Replace `src/editor/mod.rs`** (VERBATIM):

```rust
//! Editor (plan.md B2): everything is painted by hand with egui's Painter.

pub mod board;
pub mod fonts;
pub mod theme;

use crate::dsp::tuner::{Reading, Yin, BUFFER};
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use std::sync::Arc;
use std::time::Duration;

/// Editor-side state that lives as long as the window.
pub struct UiState {
    /// Param ID of the knob being dragged.
    pub drag_id: Option<&'static str>,
    /// Unsnapped normalized value during a drag.
    pub drag_norm: f32,
    /// Param ID of the momentary button held down.
    pub held_id: Option<&'static str>,
    pub yin: Yin,
    pub tuner_buf: Vec<f32>,
    pub reading: Option<Reading>,
    pub tuner_written: usize,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            drag_id: None,
            drag_norm: 0.0,
            held_id: None,
            yin: Yin::new(),
            tuner_buf: vec![0.0; BUFFER],
            reading: None,
            tuner_written: 0,
        }
    }
}

pub fn create(params: Arc<HolyMolyParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        UiState::new(),
        |ctx, _state| fonts::install(ctx),
        move |ctx, setter, state| {
            egui::CentralPanel::default().frame(egui::Frame::default()).show(ctx, |ui| {
                board::draw(ui, setter, state, &params, &shared);
            });
            ctx.request_repaint_after(Duration::from_millis(33));
        },
    )
}
```

**5. Create `src/editor/board.rs`.** This is the first version; it is replaced in S22, S24 and S25 (VERBATIM):

```rust
//! The whole editor (plan.md C4). Version 1: background, strip body, pedal bodies.

use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, pos2, vec2, Rect};

pub fn draw(ui: &mut egui::Ui, _setter: &ParamSetter, _st: &mut UiState, _params: &HolyMolyParams, _shared: &Shared) {
    let painter = ui.painter().clone();
    let o = ui.max_rect().min.to_vec2();
    let rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(pos2(x, y) + o, vec2(w, h));
    painter.rect_filled(rect(0.0, 0.0, EDITOR_W, EDITOR_H), 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            painter.rect_filled(rect(5.0 + 12.0 * i as f32, 5.0 + 12.0 * j as f32, 2.0, 2.0), 0.0, LINE);
        }
    }
    painter.rect_filled(rect(16.0, 12.0, 1088.0, 112.0), 6.0, DECK);
    for p in PEDALS {
        painter.rect_filled(rect(p.x, p.y, p.w, PEDAL_H), 6.0, p.finish.body);
    }
}
```

**6. Edit `src/lib.rs`.** Inside `impl Plugin for HolyMoly`, add this method directly below `fn params(...)`:

```rust
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.shared.clone())
    }
```

**7. Create `scripts/screenshot.ps1`** (VERBATIM). It photographs the standalone window:

```powershell
param([string]$Out = "target\screenshot.png")
cargo build --release --bin holy_moly
if ($LASTEXITCODE -ne 0) { Write-Host "SCREENSHOT FAILED: build"; exit 1 }
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class W {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
[W]::SetProcessDPIAware() | Out-Null
$p = Start-Process -FilePath "target\release\holy_moly.exe" -ArgumentList "--backend","dummy" -PassThru
Start-Sleep -Seconds 5
$p.Refresh()
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { Stop-Process -Id $p.Id -Force; Write-Host "SCREENSHOT FAILED: no window"; exit 1 }
[W]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 800
$r = New-Object W+RECT
[W]::GetWindowRect($h, [ref]$r) | Out-Null
$w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
$bmp = New-Object System.Drawing.Bitmap $w, $hh
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Stop-Process -Id $p.Id -Force
Write-Host "SCREENSHOT OK $Out ($w x $hh)"
```

**8. Create `scripts/reference.ps1`** (VERBATIM). It renders the reference design at exactly 1:1:

```powershell
$edge = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $edge) { Write-Host "REFERENCE FAILED: Edge not found"; exit 1 }
$html = (Resolve-Path "reference\holy-moly-editor.html").Path
$out = Join-Path (Resolve-Path "reference").Path "reference.png"
& $edge --headless=new --disable-gpu --hide-scrollbars --window-size=1152,792 --virtual-time-budget=5000 "--screenshot=$out" ("file:///" + $html.Replace('\','/'))
Start-Sleep -Seconds 3
if (Test-Path $out) { Write-Host "REFERENCE OK $out (editor starts at 16,16)" } else { Write-Host "REFERENCE FAILED"; exit 1 }
```

**VERIFY:**
1. CHECK → `CHECK PASSED`.
2. BUNDLE → `True`.
3. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\reference.ps1` → `REFERENCE OK`. Run this once; commit `reference\reference.png`.
4. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\screenshot.ps1` → `SCREENSHOT OK`.
5. Open `target\screenshot.png`. EXPECTED: dark dotted background, a dark strip at the top, and seven coloured rounded rectangles laid out like the reference.

If step 4 fails, see Part E, E-E3. Visual checks then become HUMAN CHECKS; continue.

If the build fails in `create_egui_editor`, see Part E, E-E1 (compare with `docs\API_NOTES.md`).

**Commit:** `S18: editor skeleton, standalone, screenshot tools`

---

### S19 — `paint.rs` (drawing helpers) and `layout.rs` (flexbox maths)

**Goal:** every later widget draws only through these helpers. Version-sensitive egui calls exist in ONE file (paint.rs), so a compile error from an egui update is fixed in one place (Part E, E-E2).

**Files:**
- Create `src/editor/paint.rs` and `src/editor/layout.rs` (VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod layout;` and `pub mod paint;` below `pub mod fonts;`.

`src/editor/paint.rs`
```rust
//! Drawing helpers (plan.md C2 glow/disabled rules, C3 text styles, D4.0 erratum).
//! All coordinates are EDITOR coordinates (0..1120, 0..760); `Canvas` adds the window origin.
//! Only egui calls that are stable across versions are used here (plan.md Part E, E-E2).

use crate::editor::theme::*;
use nih_plug_egui::egui::{self, pos2, vec2, Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Wordmark,
    PedalName,
    Note,
    Label,
    Value,
    Choice,
    Readout,
    Stage,
}

impl Style {
    pub fn family(self) -> &'static str {
        match self {
            Style::Wordmark => "display900",
            Style::PedalName | Style::Note => "display800",
            Style::Label => "mono700",
            Style::Value | Style::Stage => "mono500",
            Style::Choice | Style::Readout => "mono600",
        }
    }

    pub fn size(self) -> f32 {
        match self {
            Style::Wordmark => 40.0,
            Style::PedalName => 26.0,
            Style::Note => 52.0,
            Style::Readout => 13.0,
            _ => 10.0,
        }
    }

    /// Letter spacing in em.
    pub fn tracking(self) -> f32 {
        match self {
            Style::Wordmark => 0.02,
            Style::PedalName => 0.03,
            Style::Note => 0.0,
            Style::Label => 0.08,
            Style::Value => 0.02,
            Style::Choice => 0.06,
            Style::Readout => 0.04,
            Style::Stage => 0.10,
        }
    }

    /// plan.md D4.0: only LABEL, OPTION (Choice) and PEDAL_NAME are uppercased.
    pub fn upper(self) -> bool {
        matches!(self, Style::Label | Style::Choice | Style::PedalName)
    }

    pub fn font(self) -> FontId {
        FontId::new(self.size(), FontFamily::Name(self.family().into()))
    }
}

/// Point at angle `deg` (clockwise from 12 o'clock) on a circle. Same as `polar` in the reference.
pub fn polar(cx: f32, cy: f32, r: f32, deg: f32) -> (f32, f32) {
    let t = deg.to_radians();
    (cx + r * t.sin(), cy - r * t.cos())
}

/// Disabled rule (plan.md C2).
pub fn dim(c: Color32, disabled: bool) -> Color32 {
    if disabled {
        c.gamma_multiply(0.32)
    } else {
        c
    }
}

/// Outline points of a rounded rectangle, clockwise, 28 points.
pub fn rounded_rect_points(rect: Rect, radius: f32) -> Vec<Pos2> {
    let r = radius.min(rect.width().min(rect.height()) / 2.0);
    let r = if r < 0.0 { 0.0 } else { r };
    let corners = [
        (rect.right() - r, rect.top() + r, -90.0f32),
        (rect.right() - r, rect.bottom() - r, 0.0),
        (rect.left() + r, rect.bottom() - r, 90.0),
        (rect.left() + r, rect.top() + r, 180.0),
    ];
    let mut pts = Vec::with_capacity(28);
    for (cx, cy, start) in corners {
        for k in 0..7 {
            let a = (start + 15.0 * k as f32).to_radians();
            pts.push(pos2(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    pts
}

#[derive(Clone)]
pub struct Canvas {
    pub painter: Painter,
    pub origin: Vec2,
}

impl Canvas {
    pub fn new(ui: &egui::Ui) -> Self {
        Self { painter: ui.painter().clone(), origin: ui.max_rect().min.to_vec2() }
    }

    pub fn p(&self, x: f32, y: f32) -> Pos2 {
        pos2(x, y) + self.origin
    }

    pub fn r(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.p(x, y), vec2(w, h))
    }

    /// A canvas that only draws inside the given rectangle.
    pub fn clipped(&self, x: f32, y: f32, w: f32, h: f32) -> Canvas {
        Canvas { painter: self.painter.with_clip_rect(self.r(x, y, w, h)), origin: self.origin }
    }

    pub fn fill_rect(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color32) {
        self.painter.rect_filled(self.r(x, y, w, h), radius, color);
    }

    pub fn stroke_rect(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, width: f32, color: Color32) {
        let half = width / 2.0;
        let rect = self.r(x + half, y + half, w - width, h - width);
        let pts = rounded_rect_points(rect, (radius - half).max(0.0));
        self.painter.add(Shape::closed_line(pts, Stroke::new(width, color)));
    }

    pub fn circle(&self, cx: f32, cy: f32, r: f32, color: Color32) {
        self.painter.circle_filled(self.p(cx, cy), r, color);
    }

    pub fn ring(&self, cx: f32, cy: f32, r: f32, width: f32, color: Color32) {
        self.painter.circle_stroke(self.p(cx, cy), r, Stroke::new(width, color));
    }

    /// Glow rule (plan.md C2): a larger copy at 25 % opacity, then the shape.
    pub fn glow_circle(&self, cx: f32, cy: f32, r: f32, color: Color32) {
        self.circle(cx, cy, r + 3.0, color.gamma_multiply(0.25));
        self.circle(cx, cy, r, color);
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color32) {
        self.painter.line_segment([self.p(x1, y1), self.p(x2, y2)], Stroke::new(width, color));
    }

    pub fn glow_line(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color32) {
        self.line(x1, y1, x2, y2, width + 3.0, color.gamma_multiply(0.25));
        self.line(x1, y1, x2, y2, width, color);
    }

    pub fn polyline(&self, pts: &[(f32, f32)], width: f32, color: Color32) {
        if pts.len() < 2 {
            return;
        }
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.p(*x, *y)).collect();
        self.painter.add(Shape::line(v, Stroke::new(width, color)));
    }

    /// Filled CONVEX polygon.
    pub fn polygon(&self, pts: &[(f32, f32)], color: Color32) {
        if pts.len() < 3 {
            return;
        }
        let v: Vec<Pos2> = pts.iter().map(|(x, y)| self.p(*x, *y)).collect();
        self.painter.add(Shape::convex_polygon(v, color, Stroke::NONE));
    }

    /// Arc from `a0` to `a1` degrees (clockwise from 12 o'clock).
    pub fn arc(&self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, width: f32, color: Color32) {
        let (a0, a1) = if a1 < a0 { (a1, a0) } else { (a0, a1) };
        if a1 - a0 < 0.5 {
            return;
        }
        let n = (((a1 - a0) / 4.0).ceil() as usize).max(2);
        let pts: Vec<(f32, f32)> = (0..=n).map(|k| polar(cx, cy, r, a0 + (a1 - a0) * k as f32 / n as f32)).collect();
        self.polyline(&pts, width, color);
    }

    pub fn glow_arc(&self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, width: f32, color: Color32) {
        self.arc(cx, cy, r, a0, a1, width + 3.0, color.gamma_multiply(0.25));
        self.arc(cx, cy, r, a0, a1, width, color);
    }

    /// Dashed arc; `dash` and `gap` in px along the circle.
    pub fn dashed_arc(&self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, width: f32, color: Color32, dash: f32, gap: f32) {
        let deg_per_px = 180.0 / (std::f32::consts::PI * r.max(1.0));
        let mut a = a0.min(a1);
        let end = a0.max(a1);
        while a < end {
            let b = (a + dash * deg_per_px).min(end);
            let (x0, y0) = polar(cx, cy, r, a);
            let (x1, y1) = polar(cx, cy, r, b);
            self.line(x0, y0, x1, y1, width, color);
            a = b + gap * deg_per_px;
        }
    }

    /// Diagonal "/" stripes (CSS repeating-linear-gradient(-45deg, color 0 band, transparent band period)),
    /// clipped to the rectangle. `band` and `period` are measured across the stripes.
    pub fn stripes(&self, x: f32, y: f32, w: f32, h: f32, band: f32, period: f32, color: Color32) {
        let c = self.clipped(x, y, w, h);
        let p = period * std::f32::consts::SQRT_2;
        let b = band * std::f32::consts::SQRT_2;
        let first = -((h / p).ceil() as i32) - 1;
        let last = (w / p).ceil() as i32 + 1;
        for k in first..=last {
            let x0 = x + k as f32 * p;
            c.polygon(&[(x0, y + h), (x0 + b, y + h), (x0 + b + h, y), (x0 + h, y)], color);
        }
    }

    /// `--shadow-pedal` approximation; draw BEFORE the body.
    pub fn pedal_shadow(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        self.fill_rect(x - 1.0, y + 6.0, w + 2.0, h + 2.0, radius + 2.0, Color32::from_black_alpha(70));
        self.fill_rect(x, y + 2.0, w, h + 1.0, radius, Color32::from_black_alpha(140));
    }

    /// Top highlight and dark inner border of `--shadow-pedal`; draw AFTER the body.
    pub fn pedal_bevel(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        self.stroke_rect(x, y, w, h, radius, 1.0, Color32::from_black_alpha(89));
        self.line(x + radius, y + 1.0, x + w - radius, y + 1.0, 1.0, Color32::from_white_alpha(31));
    }

    /// `--shadow-well`: near-black inset box with a black edge.
    pub fn well(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) {
        self.fill_rect(x, y, w, h, radius, WELL);
        self.stroke_rect(x, y, w, h, radius, 1.0, Color32::BLACK);
    }

    fn glyph_widths(&self, s: &str, font: &FontId) -> (Vec<f32>, f32) {
        let mut widths = Vec::with_capacity(s.len());
        let mut height = 0.0f32;
        for ch in s.chars() {
            let g = self.painter.layout_no_wrap(ch.to_string(), font.clone(), Color32::WHITE);
            widths.push(g.size().x);
            height = height.max(g.size().y);
        }
        (widths, height)
    }

    fn prepared(&self, text: &str, style: Style) -> String {
        if style.upper() {
            text.to_uppercase()
        } else {
            text.to_owned()
        }
    }

    /// Width of `text` in `style`, including letter spacing.
    pub fn text_width(&self, text: &str, style: Style) -> f32 {
        let s = self.prepared(text, style);
        let (widths, _) = self.glyph_widths(&s, &style.font());
        let track = style.tracking() * style.size();
        widths.iter().sum::<f32>() + track * widths.len().saturating_sub(1) as f32
    }

    /// Draws `text` with letter spacing. (x, y) is the anchor point. Returns the drawn rect (window coords).
    pub fn text(&self, x: f32, y: f32, anchor: Align2, text: &str, style: Style, color: Color32) -> Rect {
        let s = self.prepared(text, style);
        let font = style.font();
        let (widths, height) = self.glyph_widths(&s, &font);
        let track = style.tracking() * style.size();
        let total = widths.iter().sum::<f32>() + track * widths.len().saturating_sub(1) as f32;
        let rect = anchor.anchor_size(self.p(x, y), vec2(total, height));
        let mut cx = rect.min.x;
        for (ch, w) in s.chars().zip(widths.iter()) {
            self.painter.text(pos2(cx, rect.min.y), Align2::LEFT_TOP, ch, font.clone(), color);
            cx += w + track;
        }
        rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_matches_the_reference() {
        let (x, y) = polar(0.0, 0.0, 10.0, 0.0);
        assert!(x.abs() < 1e-5 && (y + 10.0).abs() < 1e-5);
        let (x, y) = polar(0.0, 0.0, 10.0, 90.0);
        assert!((x - 10.0).abs() < 1e-5 && y.abs() < 1e-4);
    }

    #[test]
    fn rounded_rect_has_28_points_inside_the_rect() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 50.0));
        let pts = rounded_rect_points(rect, 6.0);
        assert_eq!(pts.len(), 28);
        for p in pts {
            assert!(p.x >= -0.01 && p.x <= 100.01 && p.y >= -0.01 && p.y <= 50.01);
        }
    }
}
```

`src/editor/layout.rs`
```rust
//! Pure layout maths (no egui), unit-tested. Mirrors the CSS flexbox rules of the reference.

/// `justify-content: space-around` with `gap`. Returns the left x of each item.
/// Negative free space or a single item behaves like `center` (CSS).
pub fn space_around(x0: f32, width: f32, items: &[f32], gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let used = items.iter().sum::<f32>() + gap * (n - 1) as f32;
    let free = width - used;
    let (start, extra) = if n == 1 || free < 0.0 {
        (x0 + free / 2.0, 0.0)
    } else {
        (x0 + free / (2.0 * n as f32), free / n as f32)
    };
    let mut out = Vec::with_capacity(n);
    let mut x = start;
    for w in items {
        out.push(x);
        x += w + gap + extra;
    }
    out
}

/// `justify-content: space-between` with `gap`. A single item sits at the start (CSS).
pub fn space_between(x0: f32, width: f32, items: &[f32], gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let used = items.iter().sum::<f32>() + gap * (n - 1) as f32;
    let extra = if n > 1 && width > used { (width - used) / (n - 1) as f32 } else { 0.0 };
    let mut out = Vec::with_capacity(n);
    let mut x = x0;
    for w in items {
        out.push(x);
        x += w + gap + extra;
    }
    out
}

/// Centred vertical stack (`flex-direction: column; justify-content: center`). Returns top y of each row.
pub fn stack_center(y0: f32, height: f32, rows: &[f32], gap: f32) -> Vec<f32> {
    let n = rows.len();
    if n == 0 {
        return Vec::new();
    }
    let used = rows.iter().sum::<f32>() + gap * (n - 1) as f32;
    let mut y = y0 + (height - used) / 2.0;
    let mut out = Vec::with_capacity(n);
    for h in rows {
        out.push(y);
        y += h + gap;
    }
    out
}

/// Items placed left to right so that the last one ends at `right`. Returns the left x of each item.
pub fn right_run(right: f32, items: &[f32], gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let used = items.iter().sum::<f32>() + gap * (n - 1) as f32;
    let mut x = right - used;
    let mut out = Vec::with_capacity(n);
    for w in items {
        out.push(x);
        x += w + gap;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_around_matches_css() {
        assert_eq!(space_around(0.0, 100.0, &[20.0, 20.0], 0.0), vec![15.0, 65.0]);
        assert_eq!(space_around(0.0, 100.0, &[20.0, 20.0], 8.0), vec![13.0, 67.0]);
        assert_eq!(space_around(10.0, 100.0, &[40.0], 8.0), vec![40.0]);
        assert_eq!(space_around(0.0, 50.0, &[40.0, 40.0], 0.0), vec![-15.0, 25.0]);
    }

    #[test]
    fn space_between_matches_css() {
        assert_eq!(space_between(0.0, 100.0, &[20.0, 20.0], 0.0), vec![0.0, 80.0]);
        assert_eq!(space_between(0.0, 100.0, &[10.0, 10.0, 10.0], 0.0), vec![0.0, 45.0, 90.0]);
        assert_eq!(space_between(5.0, 100.0, &[10.0], 0.0), vec![5.0]);
    }

    #[test]
    fn stack_center_matches_css() {
        assert_eq!(stack_center(0.0, 184.0, &[84.0, 38.0], 8.0), vec![27.0, 119.0]);
    }

    #[test]
    fn right_run_ends_at_right() {
        assert_eq!(right_run(100.0, &[20.0, 30.0], 6.0), vec![44.0, 70.0]);
    }
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `89 passed`.

If paint.rs does not compile, see Part E, E-E2. Fix ONLY inside paint.rs.

**Commit:** `S19: drawing helpers and layout maths`

---

### S20 — `src/editor/knob.rs`

**Design (read only):** this is the reference `Knob`, converted to painting.

| Part | Geometry (cap = 28 / 34 / 42 for sm / md / lg) |
|---|---|
| Dial box | `cap + 18`, centred at `(cx, top + dial/2)`. Interaction area = dial box. |
| Well | Circle of the dial box radius, WELL, 1 px black edge. |
| Track and arc | Radius `cap/2 + 4.5`, width 2.5. Track LINE from −135° to +135°. Value arc ACID with glow, from −135° (or 0° when bipolar). |
| Danger zone | DECAY dashes (1.5 / 2.5 px) from the danger angle to +135°. Above the danger angle, the arc and value turn DECAY. |
| Steps | Instead of an arc: dots (r 1.3, INK_FAINT); the lit one is r 2.2 ACID with glow. |
| Cap | 48 alternating PANEL/DECK slices of 7.5°, rotated with the value. 1 px LINE_STRONG ring. Inner PANEL disc inset 4 px. |
| Pointer | From `cap/2 − 2` inward by `0.38·cap`, width 2, ACID (DECAY when hot), with glow. Angle `−135 + 270·norm`. |
| Label | LABEL style, pedal print colour, centred, top = `top + dial + 3`. |
| Value window | WELL box, width `max(48, text + 8)`, height 14, radius 4, top = label top + 15. VALUE text in ACID (DECAY when hot). 1 px ACID outline while dragging. |
| Mouse | Vertical drag: 200 px covers the full range; Shift makes it 2000 px. Double-click resets to default. Every change is wrapped in begin/end gestures. |
| Disabled | Every colour × 0.32; no interaction. |

**Files:**
- Create `src/editor/knob.rs` (VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod knob;` below `pub mod fonts;`.

`src/editor/knob.rs`
```rust
//! Knob widget (reference: `Knob` in holy-moly-editor.html).

use crate::editor::paint::{dim, polar, Canvas, Style};
use crate::editor::theme::*;
use crate::editor::UiState;
use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui::{self, Align2, Color32, Id, Mesh, Sense, Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnobSize {
    Sm,
    Md,
    Lg,
}

impl KnobSize {
    pub fn cap(self) -> f32 {
        match self {
            KnobSize::Sm => 28.0,
            KnobSize::Md => 34.0,
            KnobSize::Lg => 42.0,
        }
    }

    pub fn dial(self) -> f32 {
        self.cap() + 18.0
    }

    /// Dial + 3 + label 12 + 3 + value window 14 (plan.md C4.3).
    pub fn height(self) -> f32 {
        self.dial() + 32.0
    }
}

/// Every knob slot is 64 px wide (plan.md B13).
pub const KNOB_SLOT_W: f32 = 64.0;

#[derive(Clone, Copy, Debug)]
pub struct KnobSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub size: KnobSize,
    pub bipolar: bool,
    /// Normalized value where the danger zone starts.
    pub danger: Option<f32>,
    /// Number of positions for stepped knobs.
    pub steps: Option<usize>,
    pub disabled: bool,
    /// Label colour (the pedal's print colour).
    pub print: Color32,
}

impl KnobSpec {
    pub const fn md(id: &'static str, label: &'static str, print: Color32) -> Self {
        Self { id, label, size: KnobSize::Md, bipolar: false, danger: None, steps: None, disabled: false, print }
    }
}

pub fn angle_of(norm: f32) -> f32 {
    -135.0 + 270.0 * norm
}

/// Snaps to the nearest step for stepped knobs.
pub fn snap(norm: f32, steps: Option<usize>) -> f32 {
    match steps {
        Some(n) if n > 1 => {
            let m = (n - 1) as f32;
            (norm * m).round() / m
        }
        _ => norm,
    }
}

fn value_arc(cv: &Canvas, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, color: Color32, disabled: bool) {
    if disabled {
        cv.arc(cx, cy, r, a0, a1, 2.5, color);
    } else {
        cv.glow_arc(cx, cy, r, a0, a1, 2.5, color);
    }
}

fn draw_cap(cv: &Canvas, cx: f32, cy: f32, cap: f32, angle: f32, disabled: bool) {
    let r = cap / 2.0;
    cv.circle(cx, cy + 2.0, r + 0.5, Color32::from_black_alpha(160));
    let mut mesh = Mesh::default();
    let c = cv.p(cx, cy);
    for k in 0..48 {
        let col = dim(if k % 2 == 0 { PANEL } else { DECK }, disabled);
        let a0 = angle + 7.5 * k as f32;
        let (x0, y0) = polar(cx, cy, r, a0);
        let (x1, y1) = polar(cx, cy, r, a0 + 7.5);
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(c, col);
        mesh.colored_vertex(cv.p(x0, y0), col);
        mesh.colored_vertex(cv.p(x1, y1), col);
        mesh.add_triangle(i, i + 1, i + 2);
    }
    cv.painter.add(Shape::mesh(mesh));
    cv.ring(cx, cy, r - 0.5, 1.0, dim(LINE_STRONG, disabled));
    cv.circle(cx, cy, r - 4.0, dim(PANEL, disabled));
    cv.arc(cx, cy, r - 4.5, -60.0, 60.0, 1.0, Color32::from_white_alpha(20));
}

/// Draws and handles one knob. `cx` = slot centre x, `top` = top of the dial box (editor coords).
pub fn knob<P: Param>(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    param: &P,
    cx: f32,
    top: f32,
    spec: KnobSpec,
) {
    let dial = spec.size.dial();
    let cap = spec.size.cap();
    let (dx, dy) = (cx, top + dial / 2.0);
    let rect = cv.r(cx - dial / 2.0, top, dial, dial);
    let sense = if spec.disabled { Sense::hover() } else { Sense::click_and_drag() };
    let resp = ui.interact(rect, Id::new(("hm_knob", spec.id)), sense);
    let mut norm = param.unmodulated_normalized_value();

    if !spec.disabled {
        if resp.double_clicked() {
            let d = param.default_normalized_value();
            setter.begin_set_parameter(param);
            setter.set_parameter_normalized(param, d);
            setter.end_set_parameter(param);
            norm = d;
        } else if resp.dragged() {
            if st.drag_id != Some(spec.id) {
                st.drag_id = Some(spec.id);
                st.drag_norm = norm;
                setter.begin_set_parameter(param);
            }
            let fine = ui.input(|i| i.modifiers.shift);
            let px = if fine { 2000.0 } else { 200.0 };
            st.drag_norm = (st.drag_norm - resp.drag_delta().y / px).clamp(0.0, 1.0);
            norm = snap(st.drag_norm, spec.steps);
            setter.set_parameter_normalized(param, norm);
        } else if st.drag_id == Some(spec.id) {
            setter.end_set_parameter(param);
            st.drag_id = None;
        }
        let _ = resp.on_hover_cursor(egui::CursorIcon::ResizeVertical);
    }

    let off = spec.disabled;
    let hot = matches!(spec.danger, Some(d) if norm > d + 1e-4);
    let accent = if hot { DECAY } else { ACID };

    cv.circle(dx, dy, dial / 2.0, WELL);
    cv.ring(dx, dy, dial / 2.0 - 0.5, 1.0, Color32::BLACK);

    let r = cap / 2.0 + 4.5;
    let av = angle_of(norm);
    match spec.steps {
        Some(n) if n > 1 => {
            cv.arc(dx, dy, r, -135.0, 135.0, 2.5, dim(LINE, off));
            let lit = (norm * (n - 1) as f32).round() as usize;
            for i in 0..n {
                let (x, y) = polar(dx, dy, r, angle_of(i as f32 / (n - 1) as f32));
                if i == lit {
                    if off {
                        cv.circle(x, y, 2.2, dim(ACID, off));
                    } else {
                        cv.glow_circle(x, y, 2.2, ACID);
                    }
                } else {
                    cv.circle(x, y, 1.3, dim(INK_FAINT, off));
                }
            }
        }
        _ => {
            let base = if spec.bipolar { 0.0 } else { -135.0 };
            match spec.danger {
                Some(d) => {
                    let ad = angle_of(d);
                    cv.arc(dx, dy, r, -135.0, ad, 2.5, dim(LINE, off));
                    cv.dashed_arc(dx, dy, r, ad, 135.0, 2.5, dim(DECAY, off), 1.5, 2.5);
                    value_arc(cv, dx, dy, r, base, av.min(ad), dim(ACID, off), off);
                    if av > ad {
                        value_arc(cv, dx, dy, r, ad, av, dim(DECAY, off), off);
                    }
                }
                None => {
                    cv.arc(dx, dy, r, -135.0, 135.0, 2.5, dim(LINE, off));
                    value_arc(cv, dx, dy, r, base, av, dim(ACID, off), off);
                }
            }
            if spec.bipolar {
                let (x, y) = polar(dx, dy, r + 3.2, 0.0);
                cv.circle(x, y, 1.0, dim(INK_FAINT, off));
            }
        }
    }

    draw_cap(cv, dx, dy, cap, av, off);

    let outer = cap / 2.0 - 2.0;
    let inner = outer - 0.38 * cap;
    let (x0, y0) = polar(dx, dy, outer, av);
    let (x1, y1) = polar(dx, dy, inner, av);
    if off {
        cv.line(x0, y0, x1, y1, 2.0, dim(accent, off));
    } else {
        cv.glow_line(x0, y0, x1, y1, 2.0, accent);
    }

    let label_top = top + dial + 3.0;
    cv.text(cx, label_top, Align2::CENTER_TOP, spec.label, Style::Label, dim(spec.print, off));

    let text = param.normalized_value_to_string(norm, true);
    let w = (cv.text_width(&text, Style::Value) + 8.0).max(48.0);
    let wy = label_top + 15.0;
    cv.well(cx - w / 2.0, wy, w, 14.0, 4.0);
    if st.drag_id == Some(spec.id) {
        cv.stroke_rect(cx - w / 2.0, wy, w, 14.0, 4.0, 1.0, ACID);
    }
    cv.text(cx, wy + 7.0, Align2::CENTER_CENTER, &text, Style::Value, dim(accent, off));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_match_the_reference() {
        assert_eq!(angle_of(0.0), -135.0);
        assert_eq!(angle_of(0.5), 0.0);
        assert_eq!(angle_of(1.0), 135.0);
    }

    #[test]
    fn sizes_match_c4_3() {
        assert_eq!(KnobSize::Sm.height(), 78.0);
        assert_eq!(KnobSize::Md.height(), 84.0);
        assert_eq!(KnobSize::Lg.height(), 92.0);
    }

    #[test]
    fn stepped_knobs_snap() {
        assert_eq!(snap(0.49, Some(3)), 0.5);
        assert_eq!(snap(0.49, None), 0.49);
        assert_eq!(snap(0.5, Some(13)), 0.5);
    }
}
```

**VERIFY:**
- CHECK → `CHECK PASSED`, with at least `92 passed`.
- BUNDLE → `True`.

The knob is first visible in S22, so there is no screenshot at this step.

> **NOTE** — If `ParamSetter`'s methods differ from the code above, check `docs\API_NOTES.md` and Part E, E-E1. Change only the call syntax. Keep the begin/set/end order.

**Commit:** `S20: knob widget`

<!-- END OF PART 4. PART 5 (Steps S21–S27, Part E troubleshooting, kickoff prompt) IS APPENDED BELOW. -->
# PART D5 — STEPS S21 TO S27 (EDITOR, VISUAL CHECK, RELEASE)

> **NOTE** — Two step names in the Part D index changed meaning. Keep the index text in PROGRESS.md as it is and do what the step headings below say.
> - S24 now builds the controls of all six on/off pedals (Drive, Fuzz, Ring, Vibe, Prism, Echo).
> - S25 builds the Looper.

---

### S21 — `src/editor/widgets.rs` (LED, footswitch, selector, toggle, push button, icons)

**Design (read only):** this file converts the reference components `Led`, `Footswitch`, `Selector`, `Toggle`, `PushButton` and `Icon` (C4.3 sizes).

| Widget | Behaviour |
|---|---|
| Footswitch, Toggle mode | A click flips the bool parameter. |
| Footswitch, Momentary mode | The parameter is true while the mouse button is held (B10). |
| Push button | Momentary, like the footswitch. |
| Selector | A click sets the enum value. |
| Every parameter change | Wrapped in begin/end gestures. |

**Files:**
- Create `src/editor/widgets.rs` (VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod widgets;` below `pub mod theme;`.

`src/editor/widgets.rs`
```rust
//! Small widgets: LED, footswitch, selector, toggle, push button and icons (reference components).

use crate::editor::paint::{dim, polar, Canvas, Style};
use crate::editor::theme::*;
use crate::editor::UiState;
use nih_plug::prelude::{BoolParam, Enum, EnumParam, ParamSetter};
use nih_plug_egui::egui::{self, Align2, Color32, Id, Mesh, Sense, Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Halo,
    Record,
    Play,
    Overdub,
    Stop,
    Clear,
    Reverse,
}

/// Filled ring sector between radii r0 and r1; angles in degrees clockwise from 12 o'clock.
pub fn sector(cv: &Canvas, cx: f32, cy: f32, r0: f32, r1: f32, a0: f32, a1: f32, color: Color32) {
    let n = (((a1 - a0).abs() / 6.0).ceil() as usize).max(1);
    let mut mesh = Mesh::default();
    for k in 0..=n {
        let a = a0 + (a1 - a0) * k as f32 / n as f32;
        let (xi, yi) = polar(cx, cy, r0, a);
        let (xo, yo) = polar(cx, cy, r1, a);
        mesh.colored_vertex(cv.p(xi, yi), color);
        mesh.colored_vertex(cv.p(xo, yo), color);
    }
    for k in 0..n as u32 {
        let i = 2 * k;
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i + 1, i + 3, i + 2);
    }
    cv.painter.add(Shape::mesh(mesh));
}

/// Draws an icon in a `size` px box whose top-left corner is (x, y). Shapes follow the 24-unit SVG paths.
pub fn icon(cv: &Canvas, which: Icon, x: f32, y: f32, size: f32, color: Color32) {
    let s = size / 24.0;
    let p = |u: f32, v: f32| (x + u * s, y + v * s);
    let (cx, cy) = p(12.0, 12.0);
    match which {
        Icon::Halo => {
            cv.ring(cx, cy, 10.6 * s, 1.2 * s, color);
            for a in [30.0, 150.0, 270.0] {
                sector(cv, cx, cy, 3.6 * s, 9.0 * s, a, a + 60.0, color);
            }
            cv.circle(cx, cy, 2.2 * s, color);
        }
        Icon::Record => cv.circle(cx, cy, 6.0 * s, color),
        Icon::Play => cv.polygon(&[p(8.0, 5.5), p(19.0, 12.0), p(8.0, 18.5)], color),
        Icon::Overdub => {
            cv.ring(cx, cy, 7.0 * s, 2.0 * s, color);
            cv.circle(cx, cy, 3.0 * s, color);
        }
        Icon::Stop => cv.polygon(&[p(6.5, 6.5), p(17.5, 6.5), p(17.5, 17.5), p(6.5, 17.5)], color),
        Icon::Clear => {
            let (a, b) = (p(6.0, 6.0), p(18.0, 18.0));
            cv.line(a.0, a.1, b.0, b.1, 2.0 * s, color);
            let (c, d) = (p(18.0, 6.0), p(6.0, 18.0));
            cv.line(c.0, c.1, d.0, d.1, 2.0 * s, color);
        }
        Icon::Reverse => {
            cv.polygon(&[p(16.0, 5.5), p(5.0, 12.0), p(16.0, 18.5)], color);
            cv.polygon(&[p(17.0, 5.5), p(19.0, 5.5), p(19.0, 18.5), p(17.0, 18.5)], color);
        }
    }
}

/// LED: 8 px dot inside a 2 px well ring. `blink` = 1 s cycle, lit for the first half.
pub fn led(ui: &egui::Ui, cv: &Canvas, cx: f32, cy: f32, on: bool, decay: bool, blink: bool) {
    let lit = on && (!blink || ui.input(|i| i.time).fract() < 0.5);
    cv.circle(cx, cy, 6.0, WELL);
    if lit {
        cv.glow_circle(cx, cy, 4.0, if decay { DECAY } else { ACID });
    } else {
        cv.circle(cx, cy, 4.0, if decay { DECAY_DEEP } else { ACID_DIM });
    }
}

/// Sets a momentary bool parameter while `down` (plan.md B10).
fn momentary(setter: &ParamSetter, st: &mut UiState, param: &BoolParam, id: &'static str, down: bool) {
    if down && st.held_id != Some(id) {
        st.held_id = Some(id);
        setter.begin_set_parameter(param);
        setter.set_parameter(param, true);
    } else if !down && st.held_id == Some(id) {
        setter.set_parameter(param, false);
        setter.end_set_parameter(param);
        st.held_id = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FootMode {
    Toggle,
    Momentary,
}

/// Footswitch centred at (cx, cy). Normal: well 52 px, cap 40 px. Small: well 34 px, cap 26 px.
pub fn footswitch(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    param: &BoolParam,
    id: &'static str,
    cx: f32,
    cy: f32,
    small: bool,
    decay: bool,
    mode: FootMode,
    lit: bool,
) {
    let (outer, cap) = if small { (17.0, 13.0) } else { (26.0, 20.0) };
    let rect = cv.r(cx - outer, cy - outer, outer * 2.0, outer * 2.0);
    let resp = ui.interact(rect, Id::new(("hm_foot", id)), Sense::click());
    let down = resp.is_pointer_button_down_on();
    match mode {
        FootMode::Toggle => {
            if resp.clicked() {
                let v = !param.value();
                setter.begin_set_parameter(param);
                setter.set_parameter(param, v);
                setter.end_set_parameter(param);
            }
        }
        FootMode::Momentary => momentary(setter, st, param, id, down),
    }
    let tone = if decay { DECAY } else { ACID };
    if lit {
        cv.circle(cx, cy, outer + 4.0, tone.gamma_multiply(0.25));
    }
    cv.circle(cx, cy, outer, WELL);
    cv.ring(cx, cy, outer - 0.5, 1.0, Color32::BLACK);
    if lit {
        cv.ring(cx, cy, outer + 0.5, 1.0, tone);
    }
    let (ccy, cr) = if down { (cy + 1.0, cap * 0.96) } else { (cy, cap) };
    cv.circle(cx, ccy + 2.0, cr + 0.5, Color32::from_black_alpha(200));
    cv.circle(cx, ccy, cr, DECK);
    cv.circle(cx, ccy, cr * 0.88, PANEL);
    cv.circle(cx, ccy, cr * 0.48, DECK);
    cv.ring(cx, ccy, cr - 0.5, 1.0, LINE_STRONG);
    cv.ring(cx, ccy, cr * 0.4, 1.0, LINE);
    cv.arc(cx, ccy, cr - 1.5, -50.0, 50.0, 1.0, Color32::from_white_alpha(36));
    let _ = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
}

/// Width of the selector box (2 px padding, 2 px gaps, options 14 px + text).
pub fn selector_width(cv: &Canvas, labels: &[&str]) -> f32 {
    4.0 + labels.iter().map(|l| 14.0 + cv.text_width(l, Style::Choice)).sum::<f32>()
        + 2.0 * labels.len().saturating_sub(1) as f32
}

/// Selector: optional LABEL at `top`, then the 22 px box 16 px lower. Returns the box width.
pub fn selector<T: Enum + PartialEq + Copy + 'static>(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    param: &EnumParam<T>,
    id: &'static str,
    options: &[(T, &str)],
    x: f32,
    top: f32,
    label: Option<&str>,
    print: Color32,
    disabled: bool,
) -> f32 {
    let labels: Vec<&str> = options.iter().map(|(_, l)| *l).collect();
    let w = selector_width(cv, &labels);
    let by = match label {
        Some(text) => {
            cv.text(x, top, Align2::LEFT_TOP, text, Style::Label, dim(print, disabled));
            top + 16.0
        }
        None => top,
    };
    cv.well(x, by, w, 22.0, 4.0);
    cv.stroke_rect(x + 1.0, by + 1.0, w - 2.0, 20.0, 3.0, 1.0, dim(LINE_STRONG, disabled));
    let current = param.value();
    let mut ox = x + 2.0;
    for (value, text) in options {
        let ow = 14.0 + cv.text_width(text, Style::Choice);
        let rect = cv.r(ox, by + 2.0, ow, 18.0);
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let resp = ui.interact(rect, Id::new(("hm_sel", id, *text)), sense);
        let on = *value == current;
        if resp.clicked() && !on && !disabled {
            setter.begin_set_parameter(param);
            setter.set_parameter(param, *value);
            setter.end_set_parameter(param);
        }
        let hover = resp.hovered() && !disabled;
        if on {
            if !disabled {
                cv.fill_rect(ox - 2.0, by, ow + 4.0, 22.0, 4.0, ACID.gamma_multiply(0.25));
            }
            cv.fill_rect(ox, by + 2.0, ow, 18.0, 2.0, dim(ACID, disabled));
        } else if hover {
            cv.fill_rect(ox, by + 2.0, ow, 18.0, 2.0, ACID_DEEP);
        }
        let ink = if on {
            ON_ACID
        } else if hover {
            ACID
        } else {
            dim(INK_MUTED, disabled)
        };
        cv.text(ox + ow / 2.0, by + 11.0, Align2::CENTER_CENTER, text, Style::Choice, ink);
        ox += ow + 2.0;
    }
    w
}

pub fn toggle_width(cv: &Canvas, text: &str) -> f32 {
    26.0 + cv.text_width(text, Style::Choice)
}

/// Toggle (22 px high) at top-left (x, y). `width` lets a column stretch it.
pub fn toggle(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    param: &BoolParam,
    id: &'static str,
    x: f32,
    y: f32,
    width: f32,
    text: &str,
    decay: bool,
) {
    let rect = cv.r(x, y, width, 22.0);
    let resp = ui.interact(rect, Id::new(("hm_toggle", id)), Sense::click());
    let on = param.value();
    if resp.clicked() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, !on);
        setter.end_set_parameter(param);
    }
    cv.well(x, y, width, 22.0, 2.0);
    cv.stroke_rect(x + 1.0, y + 1.0, width - 2.0, 20.0, 1.0, 1.0, LINE_STRONG);
    let tone = if decay { DECAY } else { ACID };
    if on {
        cv.glow_circle(x + 10.0, y + 11.0, 3.0, tone);
    } else {
        cv.circle(x + 10.0, y + 11.0, 3.0, ACID_DIM);
    }
    let ink = if on || resp.hovered() { INK } else { INK_MUTED };
    cv.text(x + 18.0, y + 11.0, Align2::LEFT_CENTER, text, Style::Choice, ink);
}

pub fn push_width(cv: &Canvas, text: &str) -> f32 {
    32.0 + cv.text_width(text, Style::Choice)
}

/// Momentary push button (22 px high) with a 12 px icon, top-left at (x, y).
pub fn push_button(
    ui: &mut egui::Ui,
    cv: &Canvas,
    setter: &ParamSetter,
    st: &mut UiState,
    param: &BoolParam,
    id: &'static str,
    x: f32,
    y: f32,
    text: &str,
    which: Icon,
) {
    let w = push_width(cv, text);
    let rect = cv.r(x, y, w, 22.0);
    let resp = ui.interact(rect, Id::new(("hm_push", id)), Sense::click());
    let down = resp.is_pointer_button_down_on();
    momentary(setter, st, param, id, down);
    cv.well(x, y, w, 22.0, 2.0);
    cv.stroke_rect(x + 1.0, y + 1.0, w - 2.0, 20.0, 1.0, 1.0, if down { ACID } else { LINE_STRONG });
    let ink = if down {
        ACID
    } else if resp.hovered() {
        INK
    } else {
        INK_MUTED
    };
    icon(cv, which, x + 7.0, y + 5.0, 12.0, ink);
    cv.text(x + 24.0, y + 11.0, Align2::LEFT_CENTER, text, Style::Choice, ink);
}
```

**VERIFY:** CHECK → `CHECK PASSED`, with at least `92 passed` (no new tests).

**Commit:** `S21: small widgets`

---

### S22 — `src/editor/strip.rs` (top strip and tuner display)

**Files:**
- Create `src/editor/strip.rs` (VERBATIM).
- Replace `src/editor/board.rs` with Version 2 (VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod strip;` below `pub mod paint;`.

`src/editor/strip.rs`
```rust
//! Top strip (plan.md C4.4) and tuner display (C4.5).

use crate::dsp::tuner::Reading;
use crate::editor::knob::{knob, KnobSize, KnobSpec};
use crate::editor::paint::{Canvas, Style};
use crate::editor::theme::*;
use crate::editor::widgets::{footswitch, icon, led, selector, FootMode, Icon};
use crate::editor::UiState;
use crate::params::{HolyMolyParams, Quality};
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, Align2};

/// Runs YIN on the newest tuner samples whenever 1024 new samples have arrived.
pub fn update_tuner(st: &mut UiState, shared: &Shared) {
    let written = shared.tuner.written();
    if written.wrapping_sub(st.tuner_written) >= 1024 {
        shared.tuner.copy_latest(&mut st.tuner_buf);
        let rate = shared.tuner.rate();
        st.reading = st.yin.detect(&st.tuner_buf, rate).map(|f| Reading::from_freq(f, 440.0));
        st.tuner_written = written;
    }
}

pub fn draw(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams, shared: &Shared) {
    update_tuner(st, shared);

    cv.pedal_shadow(16.0, 12.0, 1088.0, 112.0, 6.0);
    cv.fill_rect(16.0, 12.0, 1088.0, 112.0, 6.0, DECK);
    let bar = cv.clipped(16.0, 120.0, 1088.0, 4.0);
    bar.fill_rect(16.0, 120.0, 1088.0, 4.0, 0.0, GROUND);
    bar.stripes(16.0, 120.0, 1088.0, 4.0, 6.0, 12.0, ACID);
    cv.pedal_bevel(16.0, 12.0, 1088.0, 112.0, 6.0);

    icon(cv, Icon::Halo, 30.0, 49.0, 34.0, ACID.gamma_multiply(0.25));
    icon(cv, Icon::Halo, 32.0, 51.0, 30.0, ACID);
    cv.text(72.0, 41.0, Align2::LEFT_TOP, "HOLY MOLY", Style::Wordmark, ACID);
    cv.text(72.0, 80.0, Align2::LEFT_TOP, "STEREO PEDALBOARD · 8 STAGES", Style::Stage, INK_MUTED);

    knob(ui, cv, setter, st, &p.input, 298.0, 20.0, KnobSpec { size: KnobSize::Lg, bipolar: true, ..KnobSpec::md("in_gain", "Input", INK_MUTED) });

    tuner_display(cv, st, p);

    knob(ui, cv, setter, st, &p.tuner.reference, 822.0, 27.0, KnobSpec { size: KnobSize::Sm, ..KnobSpec::md("tun_ref", "Ref", INK_MUTED) });

    let muted = p.tuner.mute.value();
    led(ui, cv, 883.0, 38.0, muted, true, false);
    footswitch(ui, cv, setter, st, &p.tuner.mute, "tun_mute", 883.0, 64.0, true, true, FootMode::Toggle, muted);
    cv.text(883.0, 86.0, Align2::CENTER_TOP, "Mute", Style::Label, INK_MUTED);

    let opts = [
        (Quality::X1, Quality::X1.label()),
        (Quality::X2, Quality::X2.label()),
        (Quality::X4, Quality::X4.label()),
    ];
    selector(ui, cv, setter, &p.quality, "quality", &opts, 916.0, 47.0, Some("Quality"), INK_MUTED, false);

    knob(ui, cv, setter, st, &p.output, 1056.0, 20.0, KnobSpec { size: KnobSize::Lg, ..KnobSpec::md("out_gain", "Output", INK_MUTED) });
}

fn tuner_display(cv: &Canvas, st: &UiState, p: &HolyMolyParams) {
    let (x, y, w, h) = (346.0, 24.0, 432.0, 84.0);
    let muted = p.tuner.mute.value();
    let reference = p.tuner.reference.value();
    let reading = st.reading.map(|r| Reading::from_freq(r.freq, reference));
    let in_tune = matches!(reading, Some(r) if r.cents.abs() <= 5.0);

    cv.well(x, y, w, h, 4.0);
    if muted {
        let bar = cv.clipped(x, y, w, 4.0);
        bar.fill_rect(x, y, w, 4.0, 0.0, WELL);
        bar.stripes(x, y, w, 4.0, 5.0, 10.0, DECAY);
        cv.stroke_rect(x, y, w, h, 4.0, 1.0, DECAY);
    }

    // Note and octave.
    let (note, octave) = match reading {
        Some(r) => (r.name().to_string(), r.octave.to_string()),
        None => ("–".to_string(), String::new()),
    };
    if in_tune {
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            cv.text(358.0 + dx, 42.0 + dy, Align2::LEFT_TOP, &note, Style::Note, ACID.gamma_multiply(0.2));
        }
    }
    let nr = cv.text(358.0, 42.0, Align2::LEFT_TOP, &note, Style::Note, if in_tune { ACID } else { INK });
    if !octave.is_empty() {
        cv.text(nr.max.x - cv.origin.x + 2.0, 84.0, Align2::LEFT_BOTTOM, &octave, Style::Stage, INK_MUTED);
    }

    // 21-segment meter.
    let seg_w = (216.0 - 40.0) / 21.0;
    let at = reading.map(|r| (r.cents.clamp(-50.0, 50.0) / 5.0).round() as i32).unwrap_or(0);
    for i in -10i32..=10 {
        let sx = 434.0 + (i + 10) as f32 * (seg_w + 2.0);
        let (color, glow) = match reading {
            Some(_) if in_tune && i.abs() <= 1 => (ACID, true),
            Some(_) if !in_tune && i == at => (DECAY, true),
            Some(_) if !in_tune && ((at > 0 && i > 0 && i < at) || (at < 0 && i < 0 && i > at)) => (DECAY_DEEP, false),
            _ => (LINE, false),
        };
        if glow {
            cv.fill_rect(sx - 2.0, 44.0, seg_w + 4.0, 26.0, 3.0, color.gamma_multiply(0.25));
        }
        cv.fill_rect(sx, 46.0, seg_w, 22.0, 2.0, color);
        if i == 0 {
            cv.stroke_rect(sx, 46.0, seg_w, 22.0, 2.0, 1.0, INK_FAINT);
        }
    }
    cv.text(434.0, 74.0, Align2::LEFT_TOP, "b 50", Style::Stage, INK_MUTED);
    cv.text(542.0, 74.0, Align2::CENTER_TOP, "0", Style::Stage, INK_MUTED);
    cv.text(650.0, 74.0, Align2::RIGHT_TOP, "50 #", Style::Stage, INK_MUTED);

    // Side column.
    let (cents_y, ref_y) = if muted { (41.0, 61.0) } else { (50.0, 70.0) };
    let (text, color) = match reading {
        None => ("NO SIGNAL".to_string(), INK_MUTED),
        Some(_) if in_tune => ("IN TUNE".to_string(), ACID),
        Some(r) => {
            let c = r.cents.round() as i32;
            if c > 0 {
                (format!("+{c} ct #"), DECAY)
            } else {
                (format!("{c} ct b"), DECAY)
            }
        }
    };
    cv.text(766.0, cents_y, Align2::RIGHT_TOP, &text, Style::Readout, color);
    cv.text(766.0, ref_y, Align2::RIGHT_TOP, &format!("A4 = {} Hz", reference.round() as i32), Style::Stage, INK_MUTED);
    if muted {
        let tw = cv.text_width("MUTED", Style::Stage);
        cv.fill_rect(766.0 - tw - 12.0, 84.0, tw + 12.0, 14.0, 2.0, DECAY);
        cv.text(760.0, 91.0, Align2::RIGHT_CENTER, "MUTED", Style::Stage, ON_DECAY);
    }
}
```

`src/editor/board.rs` (Version 2, replace the whole file)
```rust
//! The whole editor (plan.md C4). Version 2: background, top strip, plain pedal bodies.

use crate::editor::paint::Canvas;
use crate::editor::strip;
use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

pub fn draw(ui: &mut egui::Ui, setter: &ParamSetter, st: &mut UiState, params: &HolyMolyParams, shared: &Shared) {
    let cv = Canvas::new(ui);
    background(&cv);
    strip::draw(ui, &cv, setter, st, params, shared);
    for p in PEDALS {
        cv.pedal_shadow(p.x, p.y, p.w, PEDAL_H, 6.0);
        cv.fill_rect(p.x, p.y, p.w, PEDAL_H, 6.0, p.finish.body);
    }
}

pub fn background(cv: &Canvas) {
    cv.fill_rect(0.0, 0.0, EDITOR_W, EDITOR_H, 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            cv.fill_rect(5.0 + 12.0 * i as f32, 5.0 + 12.0 * j as f32, 2.0, 2.0, 0.0, LINE);
        }
    }
}
```

**VERIFY:**
1. CHECK → `CHECK PASSED`.
2. BUNDLE → `True`.
3. Run `scripts\screenshot.ps1` and open `target\screenshot.png`. EXPECTED: the top strip matches the top of `reference\reference.png`:
   - halo icon and acid `HOLY MOLY`;
   - a large Input knob;
   - the tuner box with `–` and `NO SIGNAL`;
   - a small Ref knob;
   - the MUTE LED, footswitch and label;
   - the Quality selector with `4×` lit;
   - a large Output knob;
   - the acid/black hazard stripes along the bottom edge.

   Write any visible difference as one line in PROGRESS.md. Do NOT fix it now; S26 handles visual fixes.

**Commit:** `S22: top strip and tuner display`

---

### S23 — `src/editor/pedal.rs` (pedal frame and artwork)

**Design (read only):** body, artwork, head row and footswitch follow C4.2.

| Art | Drawing | Opacity |
|---|---|---|
| Stripes | Print colour, 7/14 px "/" stripes, y from `bottom−49` to `bottom−27`. | 1.0 |
| Halftone | 1.7 px dots on a 10 px grid in the bottom 96 px, fading out toward the top. | 0.7 |
| Diamonds | Crossing 1.5 px diagonals every 12 px in the bottom 96 px, fading out toward the top. | 0.55 |
| Rings | 6 circles, r = 34 + 12i. | 0.4 |
| Fan | 15 rays from r 34 to r 220, −84°…84°, step 12°. | 0.5 |
| Echoes | 9 bars on each side. | 0.55 |
| Orbit | Dashed circle r 33, plus a dot at the loop position. | 0.4 |

The ring, fan, echoes and orbit art are centred at (pedal centre x, `bottom − 38`) and clipped to the bottom 76 px. A bypassed pedal multiplies its art opacity by 0.18.

**Files:**
- Create `src/editor/pedal.rs` (VERBATIM).
- Replace `src/editor/board.rs` with Version 3 (VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod pedal;` below `pub mod paint;`.

`src/editor/pedal.rs`
```rust
//! Pedal frame and artwork (plan.md C4.2; reference `Pedal` and `PedalArt`).

use crate::editor::layout::stack_center;
use crate::editor::paint::{polar, Canvas, Style};
use crate::editor::theme::*;
use crate::editor::widgets::{footswitch, led, FootMode};
use crate::editor::UiState;
use nih_plug::prelude::{BoolParam, ParamSetter};
use nih_plug_egui::egui::{self, Align2};

#[derive(Clone, Copy, Debug)]
pub struct LedState {
    pub on: bool,
    pub decay: bool,
    pub blink: bool,
}

/// Body, artwork and head row. `bypassed` fades the art; `art_position` drives the Orbit dot (0..1).
pub fn frame(ui: &egui::Ui, cv: &Canvas, g: &PedalGeom, led_state: LedState, bypassed: bool, art_position: f32) {
    let (x, y, w) = (g.x, g.y, g.w);
    cv.pedal_shadow(x, y, w, PEDAL_H, 6.0);
    cv.fill_rect(x, y, w, PEDAL_H, 6.0, g.finish.body);
    let fade = if bypassed { 0.18 } else { 1.0 };
    art(&cv.clipped(x, y, w, PEDAL_H), g, fade, art_position);
    cv.pedal_bevel(x, y, w, PEDAL_H, 6.0);

    let print = g.finish.print;
    let bw = cv.text_width(g.stage, Style::Stage) + 8.0;
    cv.stroke_rect(x + 12.0, y + 19.0, bw, 14.0, 2.0, 1.0, print);
    cv.text(x + 12.0 + bw / 2.0, y + 26.0, Align2::CENTER_CENTER, g.stage, Style::Stage, print);
    cv.text(x + 12.0 + bw + 8.0, y + 26.0, Align2::LEFT_CENTER, g.name, Style::PedalName, print);
    led(ui, cv, x + w - 16.0, y + 26.0, led_state.on, led_state.decay, led_state.blink);
}

/// Standard footer: one toggle footswitch for the pedal's On parameter.
pub fn on_footswitch(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, g: &PedalGeom, param: &BoolParam, id: &'static str) {
    let on = param.value();
    footswitch(ui, cv, setter, st, param, id, g.x + g.w / 2.0, g.y + 262.0, false, false, FootMode::Toggle, on);
}

/// Top y of each row inside the body area (y+40, height 184, gap 8).
pub fn rows(g: &PedalGeom, heights: &[f32]) -> Vec<f32> {
    stack_center(g.y + 40.0, 184.0, heights, 8.0)
}

fn art(cv: &Canvas, g: &PedalGeom, fade: f32, position: f32) {
    let (x, w) = (g.x, g.w);
    let bottom = g.y + PEDAL_H;
    let print = g.finish.print;
    let col = |opacity: f32| print.gamma_multiply(opacity * fade);
    let top = bottom - 76.0;
    let (cx, cy) = (x + w / 2.0, top + 38.0);
    let svg = cv.clipped(x, top, w, 76.0);
    match g.art {
        Art::Stripes => cv.stripes(x, bottom - 49.0, w, 22.0, 7.0, 14.0, col(1.0)),
        Art::Halftone => {
            let t0 = bottom - 96.0;
            let n = (w / 10.0).ceil() as usize;
            for j in 0..10 {
                let dy = t0 + 5.0 + 10.0 * j as f32;
                let t = (bottom - dy) / 96.0;
                let m = ((1.0 - t) / 0.9).clamp(0.0, 1.0);
                for i in 0..n {
                    cv.circle(x + 5.0 + 10.0 * i as f32, dy, 1.7, col(0.7 * m));
                }
            }
        }
        Art::Diamonds => {
            let t0 = bottom - 96.0;
            let p = 12.0 * std::f32::consts::SQRT_2;
            let count = ((w + 96.0) / p).ceil() as i32 + 2;
            for band in 0..8 {
                let y0 = t0 + 12.0 * band as f32;
                let y1 = y0 + 12.0;
                let t = (bottom - (y0 + 6.0)) / 96.0;
                let m = ((1.0 - t) / 0.8).clamp(0.0, 1.0);
                if m <= 0.0 {
                    continue;
                }
                let c = col(0.55 * m);
                let clip = cv.clipped(x, y0, w, 12.0);
                for k in -10..count {
                    let s = x + k as f32 * p;
                    clip.line(s + (bottom - y1), y1, s + (bottom - y0), y0, 1.5, c);
                    clip.line(s - bottom + y1 + 96.0, y1, s - bottom + y0 + 96.0, y0, 1.5, c);
                }
            }
        }
        Art::Rings => {
            for i in 0..6 {
                svg.ring(cx, cy, 34.0 + 12.0 * i as f32, 1.5, col(0.4));
            }
        }
        Art::Fan => {
            let mut a = -84.0f32;
            while a <= 84.0 {
                let (x0, y0) = polar(cx, cy, 34.0, a);
                let (x1, y1) = polar(cx, cy, 220.0, a);
                svg.line(x0, y0, x1, y1, 1.0, col(0.5));
                a += 12.0;
            }
        }
        Art::Echoes => {
            for k in 0..9 {
                let bh = 44.0 * 0.78f32.powi(k);
                let off = 38.0 + 16.0 * k as f32;
                svg.fill_rect(cx + off - 2.0, cy - bh / 2.0, 4.0, bh, 1.0, col(0.55));
                svg.fill_rect(cx - off - 2.0, cy - bh / 2.0, 4.0, bh, 1.0, col(0.55));
            }
        }
        Art::Orbit => {
            svg.dashed_arc(cx, cy, 33.0, 0.0, 360.0, 1.5, col(0.4), 2.0, 5.0);
            let (px, py) = polar(cx, cy, 33.0, 360.0 * position);
            svg.circle(px, py, 3.0, col(0.4));
        }
    }
}
```

`src/editor/board.rs` (Version 3, replace the whole file)
```rust
//! The whole editor (plan.md C4). Version 3: background, strip, pedal frames and footswitches.

use crate::editor::paint::Canvas;
use crate::editor::pedal::{self, LedState};
use crate::editor::strip;
use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::{BoolParam, ParamSetter};
use nih_plug_egui::egui;

pub fn draw(ui: &mut egui::Ui, setter: &ParamSetter, st: &mut UiState, params: &HolyMolyParams, shared: &Shared) {
    let cv = Canvas::new(ui);
    background(&cv);
    strip::draw(ui, &cv, setter, st, params, shared);
    let ons: [(&PedalGeom, &BoolParam, &'static str); 6] = [
        (&DRIVE, &params.drive.on, "drv_on"),
        (&FUZZ, &params.fuzz.on, "fuz_on"),
        (&RING, &params.ring.on, "rng_on"),
        (&VIBE, &params.vibe.on, "vib_on"),
        (&PRISM, &params.prism.on, "prz_on"),
        (&ECHO, &params.echo.on, "ech_on"),
    ];
    for (g, param, id) in ons {
        let on = param.value();
        pedal::frame(ui, &cv, g, LedState { on, decay: false, blink: false }, !on, 0.0);
        pedal::on_footswitch(ui, &cv, setter, st, g, param, id);
    }
    pedal::frame(ui, &cv, &LOOPER, LedState { on: false, decay: false, blink: false }, false, 0.0);
}

pub fn background(cv: &Canvas) {
    cv.fill_rect(0.0, 0.0, EDITOR_W, EDITOR_H, 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            cv.fill_rect(5.0 + 12.0 * i as f32, 5.0 + 12.0 * j as f32, 2.0, 2.0, 0.0, LINE);
        }
    }
}
```

**VERIFY:**
1. CHECK → `CHECK PASSED`.
2. BUNDLE → `True`.
3. Take a screenshot. EXPECTED: all seven pedals show their art, stage badge, uppercase name, LED and footswitch. Ring and Prism show faded art and an unlit LED.

   Note differences in PROGRESS.md.

**Commit:** `S23: pedal frames and artwork`

---

### S24 — `src/editor/pedals.rs` (controls of Drive, Fuzz, Ring, Vibe, Prism, Echo)

**Design (read only):** rows follow C4.2 and C4.3.

| Rule | Detail |
|---|---|
| Knob slots | 64 px wide; `space_around` inside `x+12 … x+w−12` with an 8 px gap. Knobs sit at the bottom of their row. |
| Selector width | `max(box width, label width)`. |
| Disabled controls | Ring Freq when Mode = Self; Vibe Mix when Mode = Vibrato; Echo Time when Sync is on; Echo Division when Sync is off. |
| Echo Feedback | Danger zone starts at normalized `1.0/1.1` (100 %). |
| Echo Division | Stepped knob with 13 positions. |
| Echo row 2 | Mode selector on the left; SYNC, PING-PONG and TRAILS on the right, 6 px apart, centred vertically. |

**Files:**
- Create `src/editor/pedals.rs` (VERBATIM).
- Replace `src/editor/board.rs` with Version 4 (VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod pedals;` below `pub mod pedal;`.

`src/editor/pedals.rs`
```rust
//! Controls of the six on/off pedals (plan.md C4.2, C4.3).

use crate::editor::knob::{knob, KnobSpec, KNOB_SLOT_W};
use crate::editor::layout::{right_run, space_around};
use crate::editor::paint::{Canvas, Style};
use crate::editor::pedal::{frame, on_footswitch, rows, LedState};
use crate::editor::theme::*;
use crate::editor::widgets::{selector, selector_width, toggle, toggle_width};
use crate::editor::UiState;
use crate::params::{DriveVoice, EchoMode, HolyMolyParams, RingMode, VibeMode};
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

/// Centre x of `count` knob slots spread with space-around.
fn knob_slots(g: &PedalGeom, count: usize) -> Vec<f32> {
    let items = vec![KNOB_SLOT_W; count];
    space_around(g.x + 12.0, g.w - 24.0, &items, 8.0)
        .into_iter()
        .map(|x| x + KNOB_SLOT_W / 2.0)
        .collect()
}

fn sel_width(cv: &Canvas, labels: &[&str], label: &str) -> f32 {
    selector_width(cv, labels).max(cv.text_width(label, Style::Label))
}

fn led_on(on: bool) -> LedState {
    LedState { on, decay: false, blink: false }
}

pub fn drive(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams) {
    let g = &DRIVE;
    let d = &p.drive;
    let print = g.finish.print;
    let on = d.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 38.0]);
    let xs = knob_slots(g, 3);
    knob(ui, cv, setter, st, &d.dist, xs[0], ys[0], KnobSpec::md("drv_dist", "Dist", print));
    knob(ui, cv, setter, st, &d.tone, xs[1], ys[0], KnobSpec::md("drv_tone", "Tone", print));
    knob(ui, cv, setter, st, &d.level, xs[2], ys[0], KnobSpec::md("drv_level", "Level", print));
    let labels = ["Tight", "Classic", "Loose"];
    let sw = sel_width(cv, &labels, "Voice");
    let sx = space_around(g.x + 12.0, g.w - 24.0, &[sw], 8.0)[0];
    let opts = [(DriveVoice::Tight, labels[0]), (DriveVoice::Classic, labels[1]), (DriveVoice::Loose, labels[2])];
    selector(ui, cv, setter, &d.voice, "drv_voice", &opts, sx, ys[1], Some("Voice"), print, false);
    on_footswitch(ui, cv, setter, st, g, &d.on, "drv_on");
}

pub fn fuzz(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams) {
    let g = &FUZZ;
    let f = &p.fuzz;
    let print = g.finish.print;
    let on = f.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let xs = knob_slots(g, 2);
    knob(ui, cv, setter, st, &f.fuzz, xs[0], ys[0], KnobSpec::md("fuz_fuzz", "Fuzz", print));
    knob(ui, cv, setter, st, &f.bias, xs[1], ys[0], KnobSpec::md("fuz_bias", "Bias", print));
    knob(ui, cv, setter, st, &f.tone, xs[0], ys[1], KnobSpec::md("fuz_tone", "Tone", print));
    knob(ui, cv, setter, st, &f.volume, xs[1], ys[1], KnobSpec::md("fuz_volume", "Volume", print));
    on_footswitch(ui, cv, setter, st, g, &f.on, "fuz_on");
}

pub fn ring(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams) {
    let g = &RING;
    let r = &p.ring;
    let print = g.finish.print;
    let on = r.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let xs = knob_slots(g, 4);
    knob(ui, cv, setter, st, &r.gain, xs[0], ys[0], KnobSpec::md("rng_gain", "Gain", print));
    knob(ui, cv, setter, st, &r.dist, xs[1], ys[0], KnobSpec::md("rng_dist", "Dist", print));
    knob(ui, cv, setter, st, &r.modulation, xs[2], ys[0], KnobSpec::md("rng_mod", "Mod", print));
    knob(ui, cv, setter, st, &r.output, xs[3], ys[0], KnobSpec::md("rng_output", "Output", print));
    let labels = ["Self", "Osc"];
    let sw = sel_width(cv, &labels, "Mode");
    let pos = space_around(g.x + 12.0, g.w - 24.0, &[sw, KNOB_SLOT_W], 8.0);
    let opts = [(RingMode::SelfMod, labels[0]), (RingMode::Osc, labels[1])];
    selector(ui, cv, setter, &r.mode, "rng_mode", &opts, pos[0], ys[1] + 84.0 - 38.0, Some("Mode"), print, false);
    let self_mode = r.mode.value() == RingMode::SelfMod;
    knob(ui, cv, setter, st, &r.freq, pos[1] + KNOB_SLOT_W / 2.0, ys[1], KnobSpec { disabled: self_mode, ..KnobSpec::md("rng_freq", "Freq", print) });
    on_footswitch(ui, cv, setter, st, g, &r.on, "rng_on");
}

pub fn vibe(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams) {
    let g = &VIBE;
    let v = &p.vibe;
    let print = g.finish.print;
    let on = v.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let xs = knob_slots(g, 3);
    knob(ui, cv, setter, st, &v.rate, xs[0], ys[0], KnobSpec::md("vib_rate", "Rate", print));
    knob(ui, cv, setter, st, &v.depth, xs[1], ys[0], KnobSpec::md("vib_depth", "Depth", print));
    knob(ui, cv, setter, st, &v.feedback, xs[2], ys[0], KnobSpec::md("vib_feedback", "Feedback", print));
    let labels = ["Chorus", "Vibrato"];
    let sw = sel_width(cv, &labels, "Mode");
    let pos = space_around(g.x + 12.0, g.w - 24.0, &[KNOB_SLOT_W, sw], 8.0);
    let vibrato = v.mode.value() == VibeMode::Vibrato;
    knob(ui, cv, setter, st, &v.mix, pos[0] + KNOB_SLOT_W / 2.0, ys[1], KnobSpec { disabled: vibrato, ..KnobSpec::md("vib_mix", "Mix", print) });
    let opts = [(VibeMode::Chorus, labels[0]), (VibeMode::Vibrato, labels[1])];
    selector(ui, cv, setter, &v.mode, "vib_mode", &opts, pos[1], ys[1] + 84.0 - 38.0, Some("Mode"), print, false);
    on_footswitch(ui, cv, setter, st, g, &v.on, "vib_on");
}

pub fn prism(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams) {
    let g = &PRISM;
    let z = &p.prism;
    let print = g.finish.print;
    let on = z.on.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 84.0]);
    let a = knob_slots(g, 4);
    knob(ui, cv, setter, st, &z.pitch, a[0], ys[0], KnobSpec { bipolar: true, ..KnobSpec::md("prz_pitch", "Pitch", print) });
    knob(ui, cv, setter, st, &z.primary, a[1], ys[0], KnobSpec::md("prz_primary", "Primary", print));
    knob(ui, cv, setter, st, &z.secondary, a[2], ys[0], KnobSpec::md("prz_secondary", "Secondary", print));
    knob(ui, cv, setter, st, &z.halo, a[3], ys[0], KnobSpec::md("prz_halo", "Halo", print));
    let b = knob_slots(g, 3);
    knob(ui, cv, setter, st, &z.tracking, b[0], ys[1], KnobSpec::md("prz_tracking", "Tracking", print));
    knob(ui, cv, setter, st, &z.depth, b[1], ys[1], KnobSpec::md("prz_depth", "Depth", print));
    knob(ui, cv, setter, st, &z.tone, b[2], ys[1], KnobSpec::md("prz_tone", "Tone", print));
    on_footswitch(ui, cv, setter, st, g, &z.on, "prz_on");
}

pub fn echo(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams) {
    let g = &ECHO;
    let e = &p.echo;
    let print = g.finish.print;
    let on = e.on.value();
    let sync = e.sync.value();
    frame(ui, cv, g, led_on(on), !on, 0.0);
    let ys = rows(g, &[84.0, 38.0]);
    let xs = knob_slots(g, 6);
    knob(ui, cv, setter, st, &e.time, xs[0], ys[0], KnobSpec { disabled: sync, ..KnobSpec::md("ech_time", "Time", print) });
    knob(ui, cv, setter, st, &e.division, xs[1], ys[0], KnobSpec { steps: Some(13), disabled: !sync, ..KnobSpec::md("ech_division", "Division", print) });
    knob(ui, cv, setter, st, &e.feedback, xs[2], ys[0], KnobSpec { danger: Some(1.0 / 1.1), ..KnobSpec::md("ech_feedback", "Feedback", print) });
    knob(ui, cv, setter, st, &e.mix, xs[3], ys[0], KnobSpec::md("ech_mix", "Mix", print));
    knob(ui, cv, setter, st, &e.tone, xs[4], ys[0], KnobSpec::md("ech_tone", "Tone", print));
    knob(ui, cv, setter, st, &e.modulation, xs[5], ys[0], KnobSpec::md("ech_mod", "Mod", print));
    let opts = [
        (EchoMode::Digital, "Digital"),
        (EchoMode::Tape, "Tape"),
        (EchoMode::Multi, "Multi"),
        (EchoMode::Reverse, "Reverse"),
    ];
    selector(ui, cv, setter, &e.mode, "ech_mode", &opts, g.x + 12.0, ys[1], Some("Mode"), print, false);
    let labels = ["SYNC", "PING-PONG", "TRAILS"];
    let widths: Vec<f32> = labels.iter().map(|l| toggle_width(cv, l)).collect();
    let tx = right_run(g.x + g.w - 12.0, &widths, 6.0);
    let ty = ys[1] + 8.0;
    toggle(ui, cv, setter, &e.sync, "ech_sync", tx[0], ty, widths[0], labels[0], false);
    toggle(ui, cv, setter, &e.pingpong, "ech_pingpong", tx[1], ty, widths[1], labels[1], false);
    toggle(ui, cv, setter, &e.trails, "ech_trails", tx[2], ty, widths[2], labels[2], false);
    on_footswitch(ui, cv, setter, st, g, &e.on, "ech_on");
}
```

`src/editor/board.rs` (Version 4, replace the whole file)
```rust
//! The whole editor (plan.md C4). Version 4: everything except the Looper's controls.

use crate::editor::paint::Canvas;
use crate::editor::pedal::{self, LedState};
use crate::editor::pedals;
use crate::editor::strip;
use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

pub fn draw(ui: &mut egui::Ui, setter: &ParamSetter, st: &mut UiState, params: &HolyMolyParams, shared: &Shared) {
    let cv = Canvas::new(ui);
    background(&cv);
    strip::draw(ui, &cv, setter, st, params, shared);
    pedals::drive(ui, &cv, setter, st, params);
    pedals::fuzz(ui, &cv, setter, st, params);
    pedals::ring(ui, &cv, setter, st, params);
    pedals::vibe(ui, &cv, setter, st, params);
    pedals::prism(ui, &cv, setter, st, params);
    pedals::echo(ui, &cv, setter, st, params);
    pedal::frame(ui, &cv, &LOOPER, LedState { on: false, decay: false, blink: false }, false, 0.0);
}

pub fn background(cv: &Canvas) {
    cv.fill_rect(0.0, 0.0, EDITOR_W, EDITOR_H, 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            cv.fill_rect(5.0 + 12.0 * i as f32, 5.0 + 12.0 * j as f32, 2.0, 2.0, 0.0, LINE);
        }
    }
}
```

**VERIFY:**
1. CHECK → `CHECK PASSED`.
2. BUNDLE → `True`.
3. Take a screenshot. EXPECTED:
   - every knob shows an uppercase label in its pedal's print colour and an acid value (`50%`, `+6.0 dB`, `440 Hz`, `400 ms`, `1/4`);
   - Ring Freq and Echo Division are greyed out;
   - the end of Echo Feedback's arc is dashed orange.

   Note differences in PROGRESS.md.

**Commit:** `S24: pedal controls`

---

### S25 — `src/editor/looper_view.rs` (Looper pedal and loop display)

**Design (read only):** the reference `Looper` pedal and `LoopDisplay`.

| Item | Rule |
|---|---|
| LED | Record: orange, blinking. Overdub: orange. Play: acid. Otherwise off. |
| Loop display | 228 × 41 well. Row 1: state icon and word (Readout) on the left, time (Stage) on the right. Row 2: 4 px bar. |
| Time text | `MAX 30.0 s`; `07.3 / 30.0 s` while recording; `03.1 / 12.4 s` otherwise. Prefix `REV ` when Reverse is on. |
| Row 2 | Loop level knob, then a column: REVERSE toggle (stretched), with STOP and CLEAR push buttons below. |
| Footer | Momentary footswitch (`lop_loop`), then a `NEXT` / next-word column. The pair is centred. Next words: Empty→REC, Record→PLAY, Play→DUB, Overdub→PLAY, Stopped→PLAY. |
| Orbit dot | Follows the real loop position. |

**Files:**
- Create `src/editor/looper_view.rs` (VERBATIM).
- Replace `src/editor/board.rs` with Version 5 (the final version, VERBATIM).
- Edit `src/editor/mod.rs`: add `pub mod looper_view;` below `pub mod layout;`.

`src/editor/looper_view.rs`
```rust
//! Looper pedal and loop display (reference `Looper` pedal and `LoopDisplay`).

use crate::dsp::looper::LoopState;
use crate::editor::knob::{knob, KnobSpec, KNOB_SLOT_W};
use crate::editor::layout::space_around;
use crate::editor::paint::{Canvas, Style};
use crate::editor::pedal::{frame, rows, LedState};
use crate::editor::theme::*;
use crate::editor::widgets::{footswitch, icon, push_button, push_width, toggle, toggle_width, FootMode, Icon};
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, Align2};

fn secs(v: f32) -> String {
    if v < 10.0 {
        format!("0{v:.1}")
    } else {
        format!("{v:.1}")
    }
}

fn loop_display(cv: &Canvas, x: f32, y: f32, w: f32, state: LoopState, length_s: f32, frac: f32, reverse: bool) {
    cv.well(x, y, w, 41.0, 4.0);
    let (word, which, color) = match state {
        LoopState::Empty => ("EMPTY", Icon::Record, INK_FAINT),
        LoopState::Record => ("RECORD", Icon::Record, DECAY),
        LoopState::Play => ("PLAY", Icon::Play, ACID),
        LoopState::Overdub => ("OVERDUB", Icon::Overdub, DECAY),
        LoopState::Stopped => ("STOPPED", Icon::Stop, INK_MUTED),
    };
    icon(cv, which, x + 8.0, y + 9.0, 12.0, color);
    cv.text(x + 25.0, y + 15.0, Align2::LEFT_CENTER, word, Style::Readout, color);
    let time = match state {
        LoopState::Empty => "MAX 30.0 s".to_string(),
        LoopState::Record => format!("{} / 30.0 s", secs(length_s)),
        _ => format!("{} / {} s", secs(frac * length_s), secs(length_s)),
    };
    let time = if reverse && state != LoopState::Empty { format!("REV {time}") } else { time };
    cv.text(x + w - 8.0, y + 15.0, Align2::RIGHT_CENTER, &time, Style::Stage, INK_MUTED);
    let fill = match state {
        LoopState::Record => length_s / 30.0,
        LoopState::Empty => 0.0,
        _ => frac,
    }
    .clamp(0.0, 1.0);
    let bar_y = y + 29.0;
    cv.fill_rect(x + 8.0, bar_y, w - 16.0, 4.0, 2.0, LINE);
    let fill_color = match state {
        LoopState::Record | LoopState::Overdub => DECAY,
        LoopState::Stopped => INK_FAINT,
        _ => ACID,
    };
    if fill > 0.0 {
        cv.fill_rect(x + 8.0, bar_y, (w - 16.0) * fill, 4.0, 2.0, fill_color);
    }
}

pub fn looper(ui: &mut egui::Ui, cv: &Canvas, setter: &ParamSetter, st: &mut UiState, p: &HolyMolyParams, shared: &Shared) {
    let g = &LOOPER;
    let lp = &p.looper;
    let print = g.finish.print;
    let (state, len, pos) = shared.looper();
    let sr = shared.sample_rate.load().max(1.0);
    let frac = if len > 0 { pos as f32 / len as f32 } else { 0.0 };
    let length_s = if state == LoopState::Record { pos as f32 / sr } else { len as f32 / sr };
    let led_state = match state {
        LoopState::Record => LedState { on: true, decay: true, blink: true },
        LoopState::Overdub => LedState { on: true, decay: true, blink: false },
        LoopState::Play => LedState { on: true, decay: false, blink: false },
        _ => LedState { on: false, decay: false, blink: false },
    };
    let orbit = if state == LoopState::Empty { 0.0 } else { frac };
    frame(ui, cv, g, led_state, false, orbit);

    let ys = rows(g, &[41.0, 84.0]);
    loop_display(cv, g.x + 12.0, ys[0], g.w - 24.0, state, length_s, frac, lp.reverse.value());

    let stop_w = push_width(cv, "STOP");
    let col_w = toggle_width(cv, "REVERSE").max(stop_w + 6.0 + push_width(cv, "CLEAR"));
    let xs = space_around(g.x + 12.0, g.w - 24.0, &[KNOB_SLOT_W, col_w], 8.0);
    knob(ui, cv, setter, st, &lp.level, xs[0] + KNOB_SLOT_W / 2.0, ys[1], KnobSpec::md("lop_level", "Loop level", print));
    let col_top = ys[1] + 84.0 - 50.0;
    toggle(ui, cv, setter, &lp.reverse, "lop_reverse", xs[1], col_top, col_w, "REVERSE", false);
    push_button(ui, cv, setter, st, &lp.stop_btn, "lop_stop", xs[1], col_top + 28.0, "STOP", Icon::Stop);
    push_button(ui, cv, setter, st, &lp.clear_btn, "lop_clear", xs[1] + stop_w + 6.0, col_top + 28.0, "CLEAR", Icon::Clear);

    let next = match state {
        LoopState::Empty => "REC",
        LoopState::Record => "PLAY",
        LoopState::Play => "DUB",
        LoopState::Overdub => "PLAY",
        LoopState::Stopped => "PLAY",
    };
    let decay = matches!(state, LoopState::Record | LoopState::Overdub);
    let lit = matches!(state, LoopState::Record | LoopState::Overdub | LoopState::Play);
    let fx = g.x + g.w / 2.0 - (52.0 + 12.0 + 52.0) / 2.0 + 26.0;
    let fy = g.y + 262.0;
    footswitch(ui, cv, setter, st, &lp.loop_btn, "lop_loop", fx, fy, false, decay, FootMode::Momentary, lit);
    let tx = fx + 26.0 + 12.0;
    cv.text(tx, fy - 1.0, Align2::LEFT_BOTTOM, "NEXT", Style::Stage, print.gamma_multiply(0.8));
    cv.text(tx, fy, Align2::LEFT_TOP, next, Style::Readout, print);
}
```

`src/editor/board.rs` (Version 5, FINAL — replace the whole file)
```rust
//! The whole editor (plan.md C4). Final version.

use crate::editor::looper_view;
use crate::editor::paint::Canvas;
use crate::editor::pedals;
use crate::editor::strip;
use crate::editor::theme::*;
use crate::editor::UiState;
use crate::params::HolyMolyParams;
use crate::shared::Shared;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui;

pub fn draw(ui: &mut egui::Ui, setter: &ParamSetter, st: &mut UiState, params: &HolyMolyParams, shared: &Shared) {
    let cv = Canvas::new(ui);
    background(&cv);
    strip::draw(ui, &cv, setter, st, params, shared);
    pedals::drive(ui, &cv, setter, st, params);
    pedals::fuzz(ui, &cv, setter, st, params);
    pedals::ring(ui, &cv, setter, st, params);
    pedals::vibe(ui, &cv, setter, st, params);
    pedals::prism(ui, &cv, setter, st, params);
    pedals::echo(ui, &cv, setter, st, params);
    looper_view::looper(ui, &cv, setter, st, params, shared);
}

pub fn background(cv: &Canvas) {
    cv.fill_rect(0.0, 0.0, EDITOR_W, EDITOR_H, 0.0, GROUND);
    for i in 0..94 {
        for j in 0..64 {
            cv.fill_rect(5.0 + 12.0 * i as f32, 5.0 + 12.0 * j as f32, 2.0, 2.0, 0.0, LINE);
        }
    }
}
```

**VERIFY:**
1. CHECK → `CHECK PASSED`.
2. BUNDLE → `True`.
3. Take a screenshot. EXPECTED: the Looper shows the `EMPTY` display with `MAX 30.0 s`, the Loop level knob showing `0.0 dB`, the REVERSE / STOP / CLEAR buttons, and a footswitch with `NEXT` and `REC`.

**Commit:** `S25: looper view — editor complete`

---

### S26 — Visual review, pluginval 10, HUMAN CHECK 2

**1. Take both pictures.** Run `scripts\reference.ps1` (if `reference\reference.png` is missing) and `scripts\screenshot.ps1`.

**2. Create `docs/VISUAL_REVIEW.md`.** Look at both images and fill in this table. Write `OK` or a short description of the difference. Do not skip rows.

```markdown
# Visual review (S26)

| # | Area | Expected (from reference) | Result |
|---|---|---|---|
| 1 | Background | near-black with a faint 12 px dot grid | |
| 2 | Strip left | acid halo icon, "HOLY MOLY", grey sub-line | |
| 3 | Strip hazard bar | acid/black diagonal stripes along the bottom edge | |
| 4 | Input / Output knobs | large knobs, grey labels, acid values | |
| 5 | Tuner box | big note or "–", 21 segments, scale "b 50 · 0 · 50 #", side text | |
| 6 | Ref knob, MUTE | small knob; LED, small footswitch, "MUTE" | |
| 7 | Quality | label + 3 options, "4×" lit acid | |
| 8 | Drive | acid body, black print, black stripes art, 3 knobs, Voice selector | |
| 9 | Fuzz | black body, acid print, dot halftone fading upwards, 4 knobs | |
| 10 | Ring | orange body, faded rings (bypassed), LED off, Freq greyed | |
| 11 | Vibe | bone body, green print, diamond lattice, Mix + Mode in row 2 | |
| 12 | Prism | dark blue body, cyan print, faded fan (bypassed), 7 knobs | |
| 13 | Echo | olive body, echo bars art, 6 knobs, Mode selector, 3 toggles (TRAILS lit) | |
| 14 | Looper | dark body, dotted orbit, loop display, level knob, buttons, NEXT/REC | |
| 15 | Stage badges / names | outlined "02".."08", uppercase stencil names | |
| 16 | Footswitches | round wells with banded caps; lit pedals have an acid ring | |
```

**3. Fix rows that are not OK** (maximum TWO fix passes):
- Fix ONLY coordinates, sizes and colours, and ONLY in `src/editor/strip.rs`, `pedal.rs`, `pedals.rs`, `looper_view.rs`, `widgets.rs` or `knob.rs`.
- Never change `src/dsp/**`, `params.rs` or any test.
- After each pass, run CHECK, BUNDLE and a screenshot, then update the Result column.
- Remaining differences become HUMAN CHECK `HC-V`, which lists the row numbers.

**4. Run pluginval at strictness 10, including the GUI:**

```powershell
$pv = Get-ChildItem -Recurse tools\pluginval -Filter pluginval.exe | Select-Object -First 1
$p = Start-Process -FilePath $pv.FullName -ArgumentList '--strictness-level 10 --validate "target\bundled\Holy Moly.vst3"' -Wait -PassThru -NoNewWindow -RedirectStandardOutput target\pluginval10.log
"pluginval exit code: $($p.ExitCode)"
Get-Content target\pluginval10.log -Tail 20
```

EXPECTED: exit code 0. If it fails, see Part E, E-V1.

**5. Add HUMAN CHECK 2:**

`HC2 (S26): In Reaper, open the Holy Moly editor. Check: it looks like reference\reference.png; knobs turn by vertical drag, Shift = fine, double-click resets; selectors and toggles switch; footswitches turn pedals on/off with LEDs following; play a guitar and the tuner shows the note; Mute silences output while the tuner keeps reading; Looper: footswitch cycles REC → PLAY → DUB → PLAY, STOP and CLEAR work; MIDI notes 60/61/62 press Loop/Stop/Clear; automation of any knob moves it in the editor.`

**Commit:** `S26: visual review and pluginval 10`

---

### S27 — Release checks and final acceptance

**1. Licence file.** Download the GPL text:

```powershell
Invoke-WebRequest -UseBasicParsing -Uri "https://www.gnu.org/licenses/gpl-3.0.txt" -OutFile LICENSE
```

If the download fails, add HUMAN CHECK `HC-LIC: add the GPL-3.0 text as LICENSE`.

**2. Write `docs/USER_GUIDE.md`** (free text, 1–3 pages). Sections in this order:
1. Install (copy `Holy Moly.vst3` to `C:\Program Files\Common Files\VST3`).
2. Signal chain.
3. One short paragraph per pedal: what each knob does, taken from the concept document in `reference\`.
4. Tuner and Mute.
5. Looper and MIDI notes 60/61/62.
6. Quality and CPU.
7. Known limits: loops are not saved with the project; the chain order is fixed.

Use only the original Holy Moly names; no real brand names (concept document, "Trademarks").

**3. Write `README.md`:** what Holy Moly is, the build commands from `docs/BUILD.md`, and the licence.

**4. Edit `.gitignore`:** add `/dist`.

**5. Package:**

```powershell
cargo xtask bundle holy_moly --release
$dist = "dist\HolyMoly-0.1.0-windows"
New-Item -ItemType Directory -Force $dist | Out-Null
Copy-Item -Recurse -Force "target\bundled\Holy Moly.vst3" $dist
Copy-Item -Force LICENSE, README.md, docs\USER_GUIDE.md $dist -ErrorAction SilentlyContinue
Copy-Item -Force assets\fonts\OFL-*.txt $dist -ErrorAction SilentlyContinue
Compress-Archive -Force -Path "$dist\*" -DestinationPath "dist\HolyMoly-0.1.0-windows.zip"
Test-Path "dist\HolyMoly-0.1.0-windows.zip"
```

EXPECTED: `True`.

**6. Final acceptance checklist.** Copy it into PROGRESS.md and tick each item:

```
FINAL ACCEPTANCE
1. CHECK ......................................... CHECK PASSED, >= 92 tests, 0 failed
2. Placeholder scan (incl. SCAFFOLD) ............ clean
3. All 52 parameter IDs unchanged ............... has_52_params_with_frozen_ids passes
4. Latency ....................................... 24 samples at 1x/2x/4x (chain test)
5. pluginval strictness 10 ....................... exit code 0 (or HUMAN CHECK logged)
6. Editor ........................................ VISUAL_REVIEW.md rows OK or HC-V logged
7. Zip ........................................... dist\HolyMoly-0.1.0-windows.zip exists
8. HUMAN_CHECKS.md ............................... lists every open owner check
9. PROGRESS.md ................................... every step [x], deviations listed
```

**7. Tag the release:**

```powershell
git add -A
git commit -m "S27: release 0.1.0"
git tag v0.1.0
```

**8. STOP.** Write a final report to the owner, at most 15 lines:
- the test count;
- the pluginval result;
- the zip path;
- the open HUMAN CHECKS;
- the deviations.

---

# PART E — TROUBLESHOOTING (look up the error here BEFORE improvising)

Each entry: **symptom → action**. If nothing here matches, apply the general rule E-X1.

**E-B1. `link.exe not found`, or "linker `link.exe` not found"** → Visual C++ Build Tools are missing. Add HUMAN CHECK `Install Visual Studio Build Tools with "Desktop development with C++"`, then STOP.

**E-B2. Cargo cannot fetch the git dependency.** → Run `$env:CARGO_NET_GIT_FETCH_WITH_CLI = "true"`, then retry once. If it still fails, add a HUMAN CHECK about the network and STOP.

**E-B3. BUNDLE says "Access is denied" or "being used by another process".** → A DAW or pluginval has the plugin loaded. Add a HUMAN CHECK asking the owner to close it. Continue with CHECK-only steps.

**E-B4. "two different versions of crate `nih_plug`".** → All three git dependencies (nih_plug, nih_plug_egui, nih_plug_xtask) must have the SAME `rev`. Fix `Cargo.toml` and `xtask/Cargo.toml`.

**E-P1. `Enum` derive rejects `#[name = ...]` or `#[id = ...]`.**
1. Open `<NIH-plug source from docs\API_NOTES.md>\nih_plug_derive\src\enums.rs` and search for `"name"` and `"id"`.
2. Delete only the attribute that is not supported, on every variant.
3. Never rename variants. The editor uses `label()`, so it is unaffected.
4. Log the change in Deviations.

**E-P2. `#[nested(group = "...")]` does not compile.** → Search `nih_plug_derive\src\params.rs` for `nested`. Use the attribute form it accepts (for example `#[nested(group = "Drive")]` or `#[nested(id_prefix = "", group = "Drive")]`). IDs must stay exactly as in C1.

**E-P3. `with_value_to_string` / `with_string_to_value` type mismatch.** → Wrap the function in a closure:
- `Arc::new(|v: f32| fmt::percent(v))`
- `Arc::new(|s: &str| fmt::parse_percent(s))`

**E-E1. `create_egui_editor` arguments or closure signature mismatch.**
1. Read the signature in `docs\API_NOTES.md`.
2. Change ONLY the closure parameter lists in `src/editor/mod.rs` so their count and types match, naming unused parameters `_x`. The update body must still call `board::draw(ui, setter, state, &params, &shared)`.
3. If `ParamSetter` method names differ, rename the calls in knob.rs and widgets.rs. Keep the begin → set → end order.

**E-E2. egui drawing API differences.** Fix ONLY in the file named in the error.

| Error mentions | Fix |
|---|---|
| `rect_filled` expects `Rounding` / `CornerRadius` | `egui::Rounding::same(radius)` or `egui::CornerRadius::same(radius as u8)` |
| `convex_polygon` / `closed_line` / `line` stroke type | `Stroke::new(w, c).into()` or `egui::epaint::PathStroke::new(w, c)` |
| no method `anchor_size` | `anchor.anchor_rect(Rect::from_min_size(self.p(x, y), vec2(total, height)))` |
| no method `layout_no_wrap` on Painter | `self.painter.ctx().fonts(\|f\| f.layout_no_wrap(ch.to_string(), font.clone(), Color32::WHITE))` |
| `Mesh` not found in `egui` | `egui::epaint::Mesh` |
| `Shape::mesh` expects `Arc<Mesh>` | `Shape::mesh(std::sync::Arc::new(mesh))` |
| `FontData` into mismatch | `std::sync::Arc::new(FontData::from_static(bytes))` |
| no method `is_pointer_button_down_on` | `resp.is_pointer_button_down_on()` → `resp.hovered() && ui.input(\|i\| i.pointer.primary_down())` |
| `deprecated` warning fails clippy | put `#[allow(deprecated)]` on that one function |

**E-E3. Screenshot script fails.**
- Build error in the standalone dependencies: remove `"standalone"` from the features, delete `src/main.rs`, and make every visual check a HUMAN CHECK.
- "no window": change `Start-Sleep -Seconds 5` to `10` and retry once. Then try `--backend wasapi`.
- Black image: the GPU or OpenGL is not available to the screenshot. Make the visual checks HUMAN CHECKs.

**E-E4. Editor panics with "FontFamily::Name ... not found".** → `fonts::install` must run in the build closure of `create_egui_editor`. Check `src/editor/mod.rs`.

**E-F1. `get_fonts.ps1` reports WOFF / WOFF2 magic (`77-4F-46-46` / `77-4F-46-32`).** → Change both `-UserAgent` values to `"Mozilla/4.0"` and retry once. Otherwise follow the fallback in S17.

**E-V1. pluginval fails.**
1. Read the log and find the first `FAILED` test name.
2. Use the cause list below:
   - "parameter ... string" → a `string_to_value` does not accept its own `value_to_string` output; fix `fmt.rs` parsers (tests must still pass).
   - Crash on editor open/close → E-E1, E-E2 or E-E4.
   - Allocation assertion → E-R1.
   - Timeout → retry once with `--timeout-ms 600000`.

**E-R1. `assert_process_allocs` panic, or a host crash in debug.** → Something in `process()` allocates. Look for `format!`, `to_string`, `vec!`, `push`, `collect` or `Box::new` in `chain.rs`, `lib.rs` or `dsp/*`. Move it to `prepare`/`new`, or remove it.

**E-T1. A frozen test fails.**
1. Run `git diff -- src/dsp/util.rs src/dsp/testutil.rs`. Earlier files must be unchanged.
2. Re-copy the current step's file VERBATIM.
3. If it still fails, write the test name and the printed numbers into PROGRESS.md, mark the step `[!]` and STOP. Never edit the test.

**E-G1. `git commit` says "nothing to commit".** → Fine. Continue.

**E-X1. Anything else.**
1. Read the full error.
2. Make the smallest change inside the current step's files.
3. Re-run VERIFY. Maximum 5 attempts.
4. Then mark the step `[!]` in PROGRESS.md, write the exact error, and STOP.

---

**END OF MANUAL — HM-PLAN-002 REV A**