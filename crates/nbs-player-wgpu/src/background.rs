//! Background shaders. The shared GLSL sources use loose `uniform` declarations and
//! `gl_FragColor`; wgpu (naga) wants Vulkan-style GLSL, so wrap them: replace the uniforms
//! with one block, redirect `gl_FragColor`, and flip `gl_FragCoord` to a bottom-left origin
//! so the effects look the same as in the GL frontends.

use nbs_player_core::config::BackgroundType;
use nbs_player_core::theme::Theme;

/// Uniform block layout (std140): 3 scalars/vec2 packed in the first 16 bytes, then one
/// vec4 per theme color.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BackgroundUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub speed: f32,
    pub colors: [[f32; 4]; 7],
}

impl BackgroundUniforms {
    pub fn new(resolution: [f32; 2], time: f32, speed: f32, theme: &Theme) -> Self {
        let c = |c: nbs_player_core::types::Rgba| {
            let [r, g, b] = c.to_f32_rgb();
            [r, g, b, 1.0]
        };
        Self {
            resolution,
            time,
            speed,
            colors: [
                c(theme.background_color),
                c(theme.accent_color),
                c(theme.text_color),
                c(theme.white_key_color),
                c(theme.black_key_color),
                c(theme.white_text_key_color),
                c(theme.black_text_key_color),
            ],
        }
    }
}

const COLOR_NAMES: [&str; 7] = [
    "background_color",
    "accent_color",
    "text_color",
    "white_key_color",
    "black_key_color",
    "white_text_key_color",
    "black_text_key_color",
];

/// Vulkan-flavoured GLSL 4.50 fragment shader for `background`.
pub fn fragment_glsl(background: &BackgroundType) -> String {
    let mut out = String::from(
        "#version 450\n\
         layout(location = 0) out vec4 _fragColor;\n\
         layout(set = 0, binding = 0, std140) uniform Bg {\n\
         \x20   vec2 iResolution;\n\
         \x20   float iTime;\n\
         \x20   float speed;\n",
    );
    for name in COLOR_NAMES {
        out.push_str(&format!("    vec4 {name}_;\n"));
    }
    out.push_str("};\n");
    for name in COLOR_NAMES {
        out.push_str(&format!("#define {name} {name}_.xyz\n"));
    }
    out.push_str("vec4 _fragCoord;\n");

    for line in background.fragment_source().lines() {
        if line.trim_start().starts_with("uniform ") {
            continue;
        }
        let line = line
            .replace("gl_FragColor", "_fragColor")
            .replace("gl_FragCoord", "_fragCoord")
            .replace("void main()", "void _nbs_main()");
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(
        "void main() {\n\
         \x20   _fragCoord = vec4(gl_FragCoord.x, iResolution.y - gl_FragCoord.y, gl_FragCoord.zw);\n\
         \x20   _nbs_main();\n\
         }\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_source_has_no_loose_uniforms_or_gl_fragcolor() {
        for kind in [
            BackgroundType::Plain,
            BackgroundType::Water,
            BackgroundType::WaterFall,
            BackgroundType::Plasma,
            BackgroundType::Fire,
            BackgroundType::Grass,
            BackgroundType::Sand,
            BackgroundType::Voronoise,
        ] {
            let src = fragment_glsl(&kind);
            assert!(!src.contains("gl_FragColor"), "{kind:?}");
            assert!(!src.lines().any(|l| l.trim_start().starts_with("uniform ")), "{kind:?}");
            assert!(src.contains("_nbs_main"), "{kind:?}");
        }
    }

    #[test]
    fn uniform_struct_matches_std140_size() {
        assert_eq!(std::mem::size_of::<BackgroundUniforms>(), 16 + 7 * 16);
    }
}
