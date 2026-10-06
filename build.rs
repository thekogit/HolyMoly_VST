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
