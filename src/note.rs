use std::f64::consts::E;

use nbs_rs;

pub fn load_note_texture(
    rl: &mut raylib::RaylibHandle,
    thread: &raylib::RaylibThread,
) -> raylib::texture::Texture2D {
    let note_image_bytes = include_bytes!("../assets/note_block.png");
    let note_image = raylib::texture::Image::load_image_from_mem(".png", note_image_bytes).unwrap();
    let note_texture = rl.load_texture_from_image(thread, &note_image).unwrap();
    note_texture
}

#[derive(Clone, Debug)]
pub struct NoteBlock {
    pub was_played: bool,
    pub tick: u16,
    pub layer: u16,
    pub instrument: u8,
    pub key: u8,
    pub velocity: u8,
    pub panning: i8,
    pub pitch: i16,
}

pub fn get_note_blocks(song: &nbs_rs::NbsFile) -> Vec<Vec<NoteBlock>> {
    // Pre allocate the ticks so it doesn't have to resize the on each iteration
    let mut note_blocks: Vec<Vec<NoteBlock>> = vec![Vec::new(); song.header.song_length as usize];

    for note in &song.notes {
        let tick = note.tick as usize;
        if tick < note_blocks.len() {
            note_blocks[tick].push(NoteBlock {
                was_played: false,
                tick: note.tick,
                layer: note.layer,
                instrument: note.instrument,
                key: note.key,
                velocity: note.velocity,
                panning: note.panning,
                pitch: note.pitch,
            });
        }
    }

    if !note_blocks.iter().all(Vec::is_empty) {
        log::info!("Loaded note blocks");
    } else {
        log::warn!("No note blocks loaded");
    }

    note_blocks
}
