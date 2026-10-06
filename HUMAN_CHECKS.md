# HUMAN CHECKS — things only the owner can do

| # | From step | What to do | Result (owner fills in) |
|---|---|---|---|
| 1 | S00 | The concept file `Holy Moly — VST3 Plugin Concept.md` was not in the workspace. Copy it into `reference\` (needed for README pedal paragraphs in S27). | |
| 2 | S16 | HC1 (S16): Copy target\bundled\Holy Moly.vst3 to C:\Program Files\Common Files\VST3\. In Reaper, put Holy Moly on a guitar or drum track and use the generic parameter UI. Check: sound passes; the drive and fuzz footswitch parameters audibly change the tone; Echo repeats at 400 ms; turning Echo off with Trails on lets the repeats ring out; project save and reload keeps the settings. | |
| 3 | S18 | E-E3 (S18): Run `target\release\holy_moly.exe --backend dummy` and verify the window: dark dotted background, dark strip at the top, and seven coloured rounded rectangles laid out like the reference. | |
| 4 | S22 | E-E3 (S22): Verify top strip matches top of reference: halo icon and acid HOLY MOLY, large Input knob, tuner box with '–' and NO SIGNAL, small Ref knob, MUTE LED/footswitch/label, Quality selector with 4x lit, large Output knob, acid/black hazard stripes. | |
| 5 | S23 | E-E3 (S23): Verify all seven pedals show their art, stage badge, uppercase name, LED and footswitch. Ring and Prism show faded art and an unlit LED. | |
| 6 | S24 | E-E3 (S24): Verify Drive, Fuzz, Ring, Vibe, Prism, Echo pedal controls. Every knob shows an uppercase label in print colour and acid value; Ring Freq and Echo Division are greyed out; Echo Feedback end arc is dashed orange. | |
