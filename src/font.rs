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
) -> Result<Font, FontError> {
    let font = rl.load_font_from_memory(thread, ".ttf", bytes, 35, None)?;
    Ok(font)
}

pub fn load_fonts(id: usize, rl: &mut RaylibHandle, thread: &RaylibThread) -> Font {
    let monocraft = include_bytes!("../assets/fonts/Monocraft.ttf") as &[u8];
    let jupiterc = include_bytes!("../assets/fonts/jupiterc.ttf") as &[u8];
    let mecha = include_bytes!("../assets/fonts/Mecha.ttf") as &[u8];
    let pix_antiqua = include_bytes!("../assets/fonts/PixAntiqua.ttf") as &[u8];
    let pixelplay = include_bytes!("../assets/fonts/pixelplay.ttf") as &[u8];
    let romulus = include_bytes!("../assets/fonts/Romulus.ttf") as &[u8];
    let setbackt = include_bytes!("../assets/fonts/setbackt.ttf") as &[u8];
    let vec = vec![
        monocraft,
        jupiterc,
        mecha,
        pix_antiqua,
        pixelplay,
        romulus,
        setbackt,
    ];

    assert!(id < vec.len(), "Font ID out of bounds");

    let font_data = vec[id];

    let font = load_from_bytes(rl, thread, font_data).unwrap_or_else(|_| {
        panic!("Failed to load font from bytes: {}", id);
    });

    font
}
