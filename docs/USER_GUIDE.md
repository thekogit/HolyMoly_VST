# Holy Moly User Guide

## 1. Installation

To install Holy Moly on Windows:
1. Copy the `Holy Moly.vst3` bundle directory to the system VST3 directory:
   `C:\Program Files\Common Files\VST3\`
2. Open your DAW (such as Reaper, Cubase, Ableton Live, or Studio One) and rescan VST3 plugins if needed.
3. Holy Moly will appear under the Distortion, Delay, Modulation, and FX categories.

A standalone executable (`Holy Moly.exe`) is also included in the distribution package for standalone live performance or testing without a DAW.

---

## 2. Signal Chain

Holy Moly is a fixed-order series stereo pedalboard effect. The audio signal flows strictly from left to right through all stages:

```
[Stereo Input]
      │
      ▼
01. Top Strip Input Gain (-24 dB to +24 dB) & Tuner Tap (Decimated)
      │
      ▼
─── Oversampled Section (1×, 2×, or 4× FIR) ───
      │
      ├─► 02. Drive
      │
      ├─► 03. Fuzz
      │
      └─► 04. Ring Modulator
      │
────────────────────────────────────────────────
      │
      ▼
05. Vibe (4-Stage Optical Phase Shifter)
      │
      ▼
06. Prism (Granular Pitch Shifter)
      │
      ▼
07. Echo (Stereo Delay)
      │
      ▼
08. Looper (30-Second Stereo Sound-on-Sound)
      │
      ▼
Top Strip Output Gain & Mute Switch
      │
      ▼
[Stereo Output]
```

Every module features click-free smooth bypass switching. When bypassed, modules introduce zero unwanted colouration, and unengaged modules inside the oversampling block bypass seamlessly.

---

## 3. Pedal Descriptions and Controls

### 02. Drive
A versatile, responsive overdrive and distortion stage with asymmetric clipping.
- **Dist**: Sets the input drive and saturation intensity.
- **Tone**: A broad treble tilt filter balancing brightness and warmth.
- **Level**: Sets the output volume of the drive stage (-30 dB to +10 dB).
- **Voice**: A 3-position voicing switch selecting the low-frequency pre-clipping filter: **Tight** (high-pass at 720 Hz for punchy, modern clarity), **Classic** (300 Hz for warm vintage crunch), or **Loose** (100 Hz for full-bodied, heavy saturation).

### 03. Fuzz
A classic two-transistor Germanium/Silicon style fuzz emulation with variable bias gating.
- **Fuzz**: Controls input saturation and clipping intensity from light grit to thick sustain.
- **Bias**: Adjusts transistor bias asymmetry; lower settings produce a gated, sputty, velcro decay, while higher settings yield open, singing sustain.
- **Tone**: A gentle high-cut filter controlling top-end fizz and bite.
- **Volume**: Output level control (-30 dB to +10 dB) with ample headroom.

### 04. Ring (Ring Modulator)
A dual-mode ring modulator capable of metallic clangs, robotic overtones, and harmonic tremolo.
- **Gain**: Pre-amplification drive into the modulator core.
- **Dist**: Symmetrical soft-clipping shaper adding grit to the modulation sidebands.
- **Mod**: Balance between dry input and ring-modulated signal.
- **Output**: Master output level of the ring modulator stage.
- **Mode**: Selects between **Self** (self-modulation using rectified and filtered input audio) and **Osc** (modulation against an internal clean sine wave carrier oscillator).
- **Freq**: Carrier oscillator frequency from 10 Hz to 4000 Hz (active in Osc mode; disabled in Self mode).

### 05. Vibe
A four-stage optical phase shifter and all-pass modulation effect inspired by photocell-bulb optical circuits.
- **Rate**: LFO modulation speed from 0.1 Hz to 10.0 Hz.
- **Depth**: Intensity and sweep span of the optical bulb modulation.
- **Feedback**: Feeds filtered output back into the first stage for resonant notches.
- **Mix**: Wet/dry blend control (active in Chorus mode; disabled in Vibrato mode).
- **Mode**: Selects between **Chorus** (dry blended with phased wet signal for rich, swirling sweeps) and **Vibrato** (100% wet pitch modulation for shimmering pitch vibrato).

### 06. Prism
A granular dual-stream pitch shifter with halo octave feedback.
- **Pitch**: Chromatic transposition from −12 semitones to +12 semitones.
- **Primary**: Output level of the primary transposed grain stream.
- **Secondary**: Output level of the secondary grain stream (fixed an octave above the primary).
- **Halo**: Shimmer feedback feeding the shifted audio back through the granular buffer.
- **Tracking**: Grain window time (10 ms to 100 ms) optimizing transient response vs. smooth sustained pitch.
- **Depth**: Random pitch modulation and chorusing across granular playback heads.
- **Tone**: High-cut lowpass filter smoothing bright shifted grain artifacts.

### 07. Echo
A feature-packed stereo delay featuring multiple delay models, tempo synchronization, and self-oscillation.
- **Time**: Delay time in milliseconds (10 ms to 2000 ms, active when Sync is off).
- **Division**: Host tempo-synced musical beat division in 13 steps from 1/32 up to 1/1 dotted (active when Sync is on).
- **Feedback**: Echo repeat volume (0% to 110%). Settings above 100% engage soft-clipped analog-style self-oscillation.
- **Mix**: Wet/dry blend of delay output.
- **Tone**: High-cut damping filter simulating tape roll-off and dark analog repeats.
- **Mod**: Tape wow/flutter and digital chorus modulation depth.
- **Mode**: Selects between **Digital** (pristine repeats), **Tape** (warm saturation and filtered tails), **Multi** (three-tap rhythmic delay pattern), and **Reverse** (backwards audio repeats).
- **Sync**: Locks delay timing to host project BPM.
- **Ping-Pong**: Alternates repeat taps between left and right channels for immersive stereo width.
- **Trails**: Allows echo repeats to decay naturally when the pedal is bypassed.

### 08. Looper
A 30-second sound-on-sound stereo phrase looper located at the end of the pedalboard chain.
- **Loop Level**: Output level of the loop playback (-60 dB to +6 dB).
- **Reverse**: Plays the recorded phrase in reverse.
- **Stop**: Stops loop playback and rewinds to the start of the loop.
- **Clear**: Discards the recorded phrase and resets the looper to Empty.
- **Footswitch**: One-button performance control following the cycle:
  - *Empty* ──► **Record** (starts recording input audio)
  - *Record* ──► **Play** (closes the loop and begins playback)
  - *Play* ──► **Overdub** (adds new audio on top of existing loop)
  - *Overdub* ──► **Play** (switches back to clean playback)
  - *Stopped* ──► **Play** (starts playback from the beginning)

---

## 4. Tuner and Mute

The top strip integrates an accurate chromatic guitar and bass tuner powered by the YIN pitch-tracking algorithm:
- **Tuner Display**: Shows the detected pitch note (A to G with `#`), octave number, and cent deviation across a 21-segment precision display.
- **Ref (Reference Frequency)**: Calibrates concert pitch from 432 Hz to 448 Hz (default A4 = 440 Hz).
- **Mute**: Engaging the Mute switch silences all audio passing through the plugin's output while keeping the tuner fully active for silent tuning on stage or in the studio.

---

## 5. Looper and MIDI Control

The looper can be triggered directly from the user interface or via incoming MIDI Note messages from hardware pedal controllers, keyboards, or DAW clips:
- **MIDI Note 60 (C4)**: Triggers the **Loop** footswitch action.
- **MIDI Note 61 (C#4)**: Triggers the **Stop** button action.
- **MIDI Note 62 (D4)**: Triggers the **Clear** button action.

These MIDI notes respond on all channels, enabling immediate plug-and-play integration with MIDI footswitches and controllers.

---

## 6. Quality and CPU

The **Quality** selector on the top control strip sets the FIR oversampling factor for the non-linear distortion section (Drive, Fuzz, and Ring Modulator):
- **1×**: Base sample rate; minimum CPU consumption.
- **2×**: Double oversampling; significantly reduces aliasing harmonics.
- **4×**: Quad oversampling; ultra-clean distortion profiles with virtually zero audible aliasing folds.

### Latency
Holy Moly maintains a constant, latency-compensated delay of **24 samples** across all Quality modes (1×, 2×, and 4×). The plugin reports this fixed latency to the host at session startup so your DAW compensates automatically. Switching Quality on the fly produces zero timing jumps or phase misalignment.

---

## 7. Known Limits

- **Loop Audio Persistence**: Recorded looper buffers are stored in memory and are not serialized into DAW project files. When closing and reopening a project, the looper starts in the `EMPTY` state.
- **Fixed Signal Chain**: The pedalboard order is fixed (Drive → Fuzz → Ring → Vibe → Prism → Echo → Looper) to provide optimal gain staging and sound-shaping characteristics.
