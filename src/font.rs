use raylib::prelude::*;

#[derive(Debug)]
pub enum FontError {
    LoadError(String),
}

impl From<String> for FontError {
    fn from(err: String) -> FontError {
        FontError::LoadError(err)
    }
}

fn load_from_bytes(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bytes: &[u8],
    font_size: i32,
) -> Result<Font, FontError> {
    let font = rl.load_font_from_memory(thread, ".ttf", bytes, font_size, None)?;
    Ok(font)
}

pub fn load_fonts(id: usize, rl: &mut RaylibHandle, thread: &RaylibThread) -> Font {
    let monocraft = include_bytes!("../assets/fonts/Monocraft.ttf") as &[u8];
    let jupiterc = include_bytes!("../assets/fonts/jupiterc.ttf") as &[u8];
    let pix_antiqua = include_bytes!("../assets/fonts/PixAntiqua.ttf") as &[u8];
    let pixelplay = include_bytes!("../assets/fonts/pixelplay.ttf") as &[u8];
    let romulus = include_bytes!("../assets/fonts/Romulus.ttf") as &[u8];
    let setbackt = include_bytes!("../assets/fonts/setbackt.ttf") as &[u8];
    let available_fonts = vec![
        monocraft,   // Minecraft like
        jupiterc,    // Doom like
        pix_antiqua, // Medieval like
        pixelplay,   // Fantasy like
        romulus,     // Sci-fi like
        setbackt,    // Retro like
    ];

    let font_sizes = vec![32, 32, 32, 35, 35, 35];

    assert!(id < available_fonts.len(), "Font ID out of bounds");
    assert!(id < font_sizes.len(), "Font size ID out of bounds");

    let font_data = available_fonts[id];
    let font_size = font_sizes[id];

    let font = load_from_bytes(rl, thread, font_data, font_size).unwrap_or_else(|_| {
        panic!("Failed to load font from bytes: {}", id);
    });

    font
}
