// Standalone build for visual checks: target\release\holy_moly.exe --backend dummy
use holy_moly::HolyMoly;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<HolyMoly>();
}
