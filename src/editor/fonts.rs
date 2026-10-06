//! Font registration (plan.md C3, B14). Families: display800, display900, mono500, mono600, mono700.
//! Each family falls back to egui's default fonts for missing glyphs.

use nih_plug_egui::egui::{self, FontData, FontDefinitions, FontFamily};

pub const FAMILIES: [&str; 5] = ["display800", "display900", "mono500", "mono600", "mono700"];

#[cfg(hm_fonts)]
const FILES: [(&str, &[u8]); 5] = [
    (
        "display800",
        include_bytes!("../../assets/fonts/BigShouldersStencil-800.ttf"),
    ),
    (
        "display900",
        include_bytes!("../../assets/fonts/BigShouldersStencil-900.ttf"),
    ),
    (
        "mono500",
        include_bytes!("../../assets/fonts/MartianMono-Condensed-500.ttf"),
    ),
    (
        "mono600",
        include_bytes!("../../assets/fonts/MartianMono-Condensed-600.ttf"),
    ),
    (
        "mono700",
        include_bytes!("../../assets/fonts/MartianMono-Condensed-700.ttf"),
    ),
];

#[cfg(not(hm_fonts))]
const FILES: [(&str, &[u8]); 0] = [];

/// True when the real fonts were embedded at build time.
pub fn embedded() -> bool {
    cfg!(hm_fonts)
}

pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mono_fallback = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    let prop_fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (name, bytes) in FILES.iter() {
        fonts
            .font_data
            .insert((*name).to_owned(), FontData::from_static(bytes).into());
    }
    for name in FAMILIES {
        let mut list = Vec::new();
        if FILES.iter().any(|(n, _)| *n == name) {
            list.push(name.to_owned());
        }
        let fallback = if name.starts_with("mono") {
            &mono_fallback
        } else {
            &prop_fallback
        };
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
            assert!(
                defs.families.contains_key(&FontFamily::Name(name.into())),
                "{name}"
            );
        }
    }
}
