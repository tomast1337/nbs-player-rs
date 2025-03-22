use raylib::prelude::*;

use crate::config::ThemeConfig;

pub struct Theme {
    pub background_color: Color,
    pub accent_color: Color,
    pub text_color: Color,
    pub white_key_color: Color,
    pub black_key_color: Color,
    pub white_text_key_color: Color,
    pub black_text_key_color: Color,
}

impl Theme {
    fn convert_color(color_str: &String) -> Color {
        let color_str = color_str.trim_start_matches('#');
        let r = u8::from_str_radix(&color_str[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&color_str[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&color_str[4..6], 16).unwrap_or(0);
        Color::new(r, g, b, 255)
    }

    pub fn from_theme_config(theme_config: &ThemeConfig) -> Self {
        Theme {
            background_color: Theme::convert_color(&theme_config.background_color),
            accent_color: Theme::convert_color(&theme_config.accent_color),
            text_color: Theme::convert_color(&theme_config.text_color),
            white_key_color: Theme::convert_color(&theme_config.white_key_color),
            black_key_color: Theme::convert_color(&theme_config.black_key_color),
            white_text_key_color: Theme::convert_color(&theme_config.white_text_key_color),
            black_text_key_color: Theme::convert_color(&theme_config.black_text_key_color),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background_color: Color::SKYBLUE,
            accent_color: Color::BLUEVIOLET,
            text_color: Color::BLACK,

            white_key_color: Color::WHITE,
            black_key_color: Color::BLACK.brightness(0.2),

            white_text_key_color: Color::BLACK,
            black_text_key_color: Color::WHITE,
        }
    }
}
