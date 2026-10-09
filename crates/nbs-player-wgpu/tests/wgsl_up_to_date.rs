//! The committed WGSL must match what the converter produces from the GLSL sources.
#[path = "../tools/convert.rs"]
mod convert;

#[test]
fn generated_wgsl_is_current() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/shaders/wgsl");
    for (kind, name) in convert::ALL {
        let path = dir.join(format!("{name}_background.wgsl"));
        let on_disk = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} missing; run the convert_shaders example", path.display()));
        assert_eq!(
            on_disk,
            convert::to_wgsl(&kind),
            "{name}: stale, run `cargo run -p nbs-player-wgpu --example convert_shaders`"
        );
    }
}
