use crate::config::ThemeConfig;
use raylib::prelude::GuiControl::*;
use raylib::prelude::GuiControlProperty::*;
use raylib::prelude::*;
#[derive(Debug, Clone)]
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

    pub fn set_gui_style(&self, rl: &mut RaylibHandle) {
        let base_w = Color::WHITE.alpha(0.).color_to_int();
        let accent_color = self.accent_color.color_to_int();
        let background_color = self.background_color.color_to_int();
        let black_key_color = self.black_key_color.color_to_int();
        // ------------------------------BUTTON STYLE------------------------------
        rl.gui_set_style(BUTTON, BASE_COLOR_NORMAL, base_w);
        rl.gui_set_style(BUTTON, BASE_COLOR_FOCUSED, base_w);
        rl.gui_set_style(BUTTON, BASE_COLOR_PRESSED, base_w);
        rl.gui_set_style(BUTTON, TEXT_COLOR_NORMAL, base_w);
        rl.gui_set_style(BUTTON, BORDER_COLOR_NORMAL, base_w);
        rl.gui_set_style(BUTTON, BORDER_COLOR_PRESSED, accent_color);
        rl.gui_set_style(BUTTON, BORDER_COLOR_FOCUSED, accent_color);
        // ------------------------------SLIDER STYLE------------------------------
        rl.gui_set_style(SLIDER, BASE_COLOR_NORMAL, background_color);
        rl.gui_set_style(SLIDER, BASE_COLOR_FOCUSED, black_key_color);
        rl.gui_set_style(
            SLIDER,
            BASE_COLOR_PRESSED,
            self.white_key_color.brightness(0.9).color_to_int(),
        );
        rl.gui_set_style(SLIDER, BORDER_COLOR_NORMAL, base_w);
        rl.gui_set_style(SLIDER, BORDER_COLOR_PRESSED, accent_color);
        rl.gui_set_style(SLIDER, BORDER_COLOR_FOCUSED, accent_color);
        rl.gui_set_style(
            SLIDER,
            TEXT_COLOR_NORMAL,
            self.white_key_color.alpha(1.).color_to_int(),
        );
        rl.gui_set_style(SLIDER, TEXT_COLOR_FOCUSED, self.accent_color.color_to_int());
        rl.gui_set_style(SLIDER, TEXT_COLOR_PRESSED, self.accent_color.color_to_int());
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
