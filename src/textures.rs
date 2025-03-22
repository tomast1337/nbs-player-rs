use raylib::prelude::*;

pub struct Textures {
    pub note_texture: Texture2D,
    pub piano_key_texture: Texture2D,
    pub play_button: Texture2D,
    pub pause_button: Texture2D,
    pub reset_button: Texture2D,
    pub volume_plus_button: Texture2D,
    pub volume_minus_button: Texture2D,
    pub fullscreen_button: Texture2D,
    pub timeline_pill: Texture2D,
}

fn load_from_mem(
    rl: &mut raylib::RaylibHandle,
    thread: &raylib::RaylibThread,
    data: &[u8],
    file_type: &str,
) -> Texture2D {
    let image = raylib::texture::Image::load_image_from_mem(file_type, data).unwrap();
    let texture = rl.load_texture_from_image(thread, &image).unwrap();
    texture.set_texture_filter(thread, raylib::consts::TextureFilter::TEXTURE_FILTER_POINT);
    texture
}

pub fn load_textures(rl: &mut raylib::RaylibHandle, thread: &raylib::RaylibThread) -> Textures {
    let piano_key_texture = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/key_grey.png"),
        ".png",
    );
    let note_texture = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/note_block.png"),
        ".png",
    );

    let play_button = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/play.png"),
        ".png",
    );

    let pause_button = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/pause.png"),
        ".png",
    );

    let reset_button = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/reset.png"),
        ".png",
    );

    let volume_plus_button = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/volume_plus.png"),
        ".png",
    );

    let volume_minus_button = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/volume_minus.png"),
        ".png",
    );

    let fullscreen_button = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/fullscreen.png"),
        ".png",
    );

    let timeline_pill = load_from_mem(
        rl,
        thread,
        include_bytes!("../assets/textures/pill.png"),
        ".png",
    );

    Textures {
        note_texture,
        piano_key_texture,
        play_button,
        pause_button,
        reset_button,
        volume_plus_button,
        volume_minus_button,
        fullscreen_button,
        timeline_pill,
    }
}
