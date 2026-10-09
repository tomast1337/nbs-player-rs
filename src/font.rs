use raylib::prelude::*;

#[derive(Debug)]
pub enum FontError {
    LoadError(),
}

impl From<String> for FontError {
    fn from(_err: String) -> FontError {
        FontError::LoadError()
    }
}

#[inline]
fn load_from_bytes(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bytes: &[u8],
    font_size: i32,
) -> Result<Font, FontError> {
    let font = match rl.load_font_from_memory(thread, ".ttf", bytes, font_size, None) {
        Err(_) => {
            log::error!("Failed to load font from memory");
            return Err(FontError::LoadError());
        }
        Ok(font) => {
            log::info!("Font loaded successfully");
            font
        }
    };
    Ok(font)
}

use nbs_player_core::config::FontID;

pub fn load_fonts(id: FontID, rl: &mut RaylibHandle, thread: &RaylibThread) -> Font {
    load_from_bytes(rl, thread, id.ttf_bytes(), 64).unwrap()
}
