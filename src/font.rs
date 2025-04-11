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

use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FontID {
    Monocraft,  // Minecraft like
    JupiterC,   // Doom like
    PixAntiqua, // Medieval like
    PixelPlay,  // Fantasy like
    Romulus,    // Sci-fi like
    Setbackt,   // Retro like
}

pub fn load_fonts(id: FontID, rl: &mut RaylibHandle, thread: &RaylibThread) -> Font {
    let monocraft = include_bytes!("../assets/fonts/Monocraft.ttf") as &[u8];
    let jupiterc = include_bytes!("../assets/fonts/jupiterc.ttf") as &[u8];
    let pix_antiqua = include_bytes!("../assets/fonts/PixAntiqua.ttf") as &[u8];
    let pixelplay = include_bytes!("../assets/fonts/pixelplay.ttf") as &[u8];
    let romulus = include_bytes!("../assets/fonts/Romulus.ttf") as &[u8];
    let setbackt = include_bytes!("../assets/fonts/setbackt.ttf") as &[u8];
    match id {
        FontID::Monocraft => load_from_bytes(rl, thread, monocraft, 64).unwrap(),
        FontID::JupiterC => load_from_bytes(rl, thread, jupiterc, 64).unwrap(),
        FontID::PixAntiqua => load_from_bytes(rl, thread, pix_antiqua, 64).unwrap(),
        FontID::PixelPlay => load_from_bytes(rl, thread, pixelplay, 64).unwrap(),
        FontID::Romulus => load_from_bytes(rl, thread, romulus, 64).unwrap(),
        FontID::Setbackt => load_from_bytes(rl, thread, setbackt, 64).unwrap(),
    }
}
