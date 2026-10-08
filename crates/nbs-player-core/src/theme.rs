use crate::config::ThemeConfig;
use crate::types::Rgba;

#[derive(Debug, Clone)]
pub struct Theme {
    pub background_color: Rgba,
    pub accent_color: Rgba,
    pub text_color: Rgba,
    pub white_key_color: Rgba,
    pub black_key_color: Rgba,
    pub white_text_key_color: Rgba,
    pub black_text_key_color: Rgba,
}

impl Theme {
    fn convert_color(s: &str) -> Rgba {
        Rgba::from_hex(s).unwrap_or_else(|| {
            log::warn!("Invalid theme color {:?}, using black", s);
            Rgba::BLACK
        })
    }

    pub fn from_theme_config(c: &ThemeConfig) -> Self {
        Theme {
            background_color: Self::convert_color(&c.background_color),
            accent_color: Self::convert_color(&c.accent_color),
            text_color: Self::convert_color(&c.text_color),
            white_key_color: Self::convert_color(&c.white_key_color),
            black_key_color: Self::convert_color(&c.black_key_color),
            white_text_key_color: Self::convert_color(&c.white_text_key_color),
            black_text_key_color: Self::convert_color(&c.black_text_key_color),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background_color: Rgba::new(102, 191, 255, 255), // sky blue
            accent_color: Rgba::new(138, 43, 226, 255),      // blue violet
            text_color: Rgba::BLACK,
            white_key_color: Rgba::WHITE,
            black_key_color: Rgba::BLACK.brightness(0.2),
            white_text_key_color: Rgba::BLACK,
            black_text_key_color: Rgba::WHITE,
        }
    }
}
