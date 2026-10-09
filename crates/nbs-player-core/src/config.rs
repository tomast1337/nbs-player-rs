use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum BackgroundType {
    Plain,
    Water,
    WaterFall,
    Plasma,
    Fire,
    Grass,
    Sand,
    Voronoise,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FontID {
    Monocraft,  // Minecraft like
    JupiterC,   // Doom like
    PixAntiqua, // Medieval like
    PixelPlay,  // Fantasy like
    Romulus,    // Sci-fi like
    Setbackt,   // Retro like
}

impl FontID {
    /// The bundled TrueType font data.
    pub fn ttf_bytes(&self) -> &'static [u8] {
        match self {
            FontID::Monocraft => include_bytes!("../../../assets/fonts/Monocraft.ttf"),
            FontID::JupiterC => include_bytes!("../../../assets/fonts/jupiterc.ttf"),
            FontID::PixAntiqua => include_bytes!("../../../assets/fonts/PixAntiqua.ttf"),
            FontID::PixelPlay => include_bytes!("../../../assets/fonts/pixelplay.ttf"),
            FontID::Romulus => include_bytes!("../../../assets/fonts/Romulus.ttf"),
            FontID::Setbackt => include_bytes!("../../../assets/fonts/setbackt.ttf"),
        }
    }
}

impl BackgroundType {
    /// Animation speed multiplier fed to the shader's `speed` uniform.
    pub fn speed(&self) -> f32 {
        match self {
            BackgroundType::Plain => 0.0,
            BackgroundType::Water => 2.5,
            BackgroundType::WaterFall => 0.5,
            BackgroundType::Plasma => 0.012,
            BackgroundType::Fire => 1.0,
            BackgroundType::Grass => 0.4,
            BackgroundType::Sand => 0.2,
            BackgroundType::Voronoise => 0.1,
        }
    }

    /// GLSL fragment shader body (GLSL 1.00 style, no header). It expects the uniforms
    /// `iResolution`, `iTime`, `speed` and the seven theme colors (`background_color`, ...).
    pub fn fragment_source(&self) -> &'static str {
        match self {
            BackgroundType::Plain => include_str!("../../../assets/shaders/plain_background.frag"),
            BackgroundType::Water => include_str!("../../../assets/shaders/water_background.frag"),
            BackgroundType::WaterFall => {
                include_str!("../../../assets/shaders/water_fall_background.frag")
            }
            BackgroundType::Plasma => include_str!("../../../assets/shaders/plasma_background.frag"),
            BackgroundType::Fire => include_str!("../../../assets/shaders/fire_background.frag"),
            BackgroundType::Grass => include_str!("../../../assets/shaders/grass_background.frag"),
            BackgroundType::Sand => include_str!("../../../assets/shaders/sand_background.frag"),
            BackgroundType::Voronoise => {
                include_str!("../../../assets/shaders/voronoise_background.frag")
            }
        }
    }
}

/// Which audio implementation plays the notes.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioBackendKind {
    /// raylib `Sound` objects (original behavior).
    #[default]
    Raylib,
    /// Software mixer fed to a cpal output stream.
    Cpal,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThemeConfig {
    pub background_color: String,
    pub accent_color: String,
    pub text_color: String,
    pub white_key_color: String,
    pub black_key_color: String,
    pub white_text_key_color: String,
    pub black_text_key_color: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub font_id: FontID,
    pub background: BackgroundType,
    pub window_width: u32,
    pub window_height: u32,
    pub theme: ThemeConfig,
    pub initial_volume: Option<f32>,
    pub target_fps: Option<u32>,
    #[serde(default)]
    pub audio_backend: AudioBackendKind,
}

impl AppConfig {
    /// A ready-to-run 1280x720 configuration for frontends started without one.
    pub fn demo() -> Self {
        AppConfig {
            font_id: FontID::PixelPlay,
            background: BackgroundType::Plain,
            window_width: 1280,
            window_height: 720,
            theme: ThemeConfig {
                background_color: "#66BFFF".into(),
                accent_color: "#8A2BE2".into(),
                text_color: "#000000".into(),
                white_key_color: "#FFFFFF".into(),
                black_key_color: "#333333".into(),
                white_text_key_color: "#000000".into(),
                black_text_key_color: "#FFFFFF".into(),
            },
            initial_volume: None,
            target_fps: None,
            audio_backend: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_background_has_a_shader_with_the_expected_uniforms() {
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
            let src = kind.fragment_source();
            assert!(src.contains("void main"), "{kind:?}");
            assert!(src.contains("background_color"), "{kind:?}");
        }
    }

    #[test]
    fn audio_backend_defaults_to_raylib_when_omitted() {
        let cfg: AppConfig = serde_json::from_str(
            r##"{"font_id":"PixelPlay","background":"Plain","window_width":800,"window_height":600,
            "theme":{"background_color":"#000000","accent_color":"#000000","text_color":"#000000",
            "white_key_color":"#000000","black_key_color":"#000000","white_text_key_color":"#000000",
            "black_text_key_color":"#000000"}}"##,
        )
        .unwrap();
        assert_eq!(cfg.audio_backend, AudioBackendKind::Raylib);
    }
}
