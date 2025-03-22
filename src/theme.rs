use raylib::prelude::*;
pub struct Theme {
    pub background_color: Color,
    pub accent_color: Color,
    pub white_key_color: Color,
    pub black_key_color: Color,
    pub text_color: Color,
    pub white_text_key_color: Color,
    pub black_text_key_color: Color,
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
