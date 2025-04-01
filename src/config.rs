use serde::{Deserialize, Serialize};

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
    pub font_id: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub theme: ThemeConfig,
}
