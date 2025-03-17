use std::sync::OnceLock;

use macroquad::text::Font;

pub static FONT: OnceLock<Font> = OnceLock::new();
