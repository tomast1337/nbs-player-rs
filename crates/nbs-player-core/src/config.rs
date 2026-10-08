use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
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
