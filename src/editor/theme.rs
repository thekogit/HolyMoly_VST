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

pub const FINISH_ACID: Finish = Finish {
    body: hex(0xe4ff1a),
    print: hex(0x0a0c07),
};
pub const FINISH_BLACK: Finish = Finish {
    body: hex(0x161912),
    print: hex(0xe4ff1a),
};
pub const FINISH_DECAY: Finish = Finish {
    body: hex(0xff6b1a),
    print: hex(0x0a0c07),
};
pub const FINISH_BONE: Finish = Finish {
    body: hex(0xe4e8cc),
    print: hex(0x1d5a2a),
};
pub const FINISH_CHERENKOV: Finish = Finish {
    body: hex(0x0a2431),
    print: hex(0x47d9ff),
};
pub const FINISH_OLIVE: Finish = Finish {
    body: hex(0x394020),
    print: hex(0xe4ff1a),
};
pub const FINISH_LEAD: Finish = Finish {
    body: hex(0x262b1e),
    print: hex(0xe4ff1a),
};

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

pub const DRIVE: PedalGeom = PedalGeom {
    name: "Drive",
    stage: "02",
    x: 16.0,
    y: 136.0,
    w: 240.0,
    finish: FINISH_ACID,
    art: Art::Stripes,
};
pub const FUZZ: PedalGeom = PedalGeom {
    name: "Fuzz",
    stage: "03",
    x: 268.0,
    y: 136.0,
    w: 232.0,
    finish: FINISH_BLACK,
    art: Art::Halftone,
};
pub const RING: PedalGeom = PedalGeom {
    name: "Ring",
    stage: "04",
    x: 512.0,
    y: 136.0,
    w: 316.0,
    finish: FINISH_DECAY,
    art: Art::Rings,
};
pub const VIBE: PedalGeom = PedalGeom {
    name: "Vibe",
    stage: "05",
    x: 840.0,
    y: 136.0,
    w: 264.0,
    finish: FINISH_BONE,
    art: Art::Diamonds,
};
pub const PRISM: PedalGeom = PedalGeom {
    name: "Prism",
    stage: "06",
    x: 16.0,
    y: 448.0,
    w: 336.0,
    finish: FINISH_CHERENKOV,
    art: Art::Fan,
};
pub const ECHO: PedalGeom = PedalGeom {
    name: "Echo",
    stage: "07",
    x: 364.0,
    y: 448.0,
    w: 476.0,
    finish: FINISH_OLIVE,
    art: Art::Echoes,
};
pub const LOOPER: PedalGeom = PedalGeom {
    name: "Looper",
    stage: "08",
    x: 852.0,
    y: 448.0,
    w: 252.0,
    finish: FINISH_LEAD,
    art: Art::Orbit,
};

pub const PEDALS: [PedalGeom; 7] = [DRIVE, FUZZ, RING, VIBE, PRISM, ECHO, LOOPER];
