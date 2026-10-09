use nbs_player_core::render::Sprite;
use raylib::prelude::*;

#[derive(Debug)]
pub struct Textures {
    pub note_texture: Texture2D,
    pub piano_key_texture: Texture2D,
    pub play_button: Texture2D,
    pub pause_button: Texture2D,
    pub reset_button: Texture2D,
    pub fullscreen_button: Texture2D,
    pub vol_000: Texture2D,
    pub vol_025: Texture2D,
    pub vol_050: Texture2D,
    pub vol_075: Texture2D,
    pub vol_100: Texture2D,
}

fn load(rl: &mut RaylibHandle, thread: &RaylibThread, sprite: Sprite) -> Texture2D {
    let image = Image::load_image_from_mem(".png", sprite.png_bytes()).unwrap();
    let texture = rl.load_texture_from_image(thread, &image).unwrap();
    texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
    texture
}

pub fn load_textures(rl: &mut RaylibHandle, thread: &RaylibThread) -> Textures {
    Textures {
        note_texture: load(rl, thread, Sprite::Note),
        piano_key_texture: load(rl, thread, Sprite::PianoKey),
        play_button: load(rl, thread, Sprite::Play),
        pause_button: load(rl, thread, Sprite::Pause),
        reset_button: load(rl, thread, Sprite::Reset),
        fullscreen_button: load(rl, thread, Sprite::Fullscreen),
        vol_000: load(rl, thread, Sprite::Volume0),
        vol_025: load(rl, thread, Sprite::Volume25),
        vol_050: load(rl, thread, Sprite::Volume50),
        vol_075: load(rl, thread, Sprite::Volume75),
        vol_100: load(rl, thread, Sprite::Volume100),
    }
}
