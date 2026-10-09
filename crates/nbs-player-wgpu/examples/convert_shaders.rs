//! Regenerate `assets/shaders/wgsl/*.wgsl` from the shared GLSL sources.
//!   cargo run -p nbs-player-wgpu --example convert_shaders
#[path = "../tools/convert.rs"]
mod convert;

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/shaders/wgsl");
    std::fs::create_dir_all(&dir).unwrap();
    for (kind, name) in convert::ALL {
        let path = dir.join(format!("{name}_background.wgsl"));
        std::fs::write(&path, convert::to_wgsl(&kind)).unwrap();
        println!("wrote {}", path.display());
    }
}
