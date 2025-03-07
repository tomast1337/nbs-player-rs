use bevy::prelude::*;
use nbs_rs::Note;

use crate::{
    piano::{self, PianoKey},
    AppState,
};

static NOTE_TEXTURE: &[u8] = include_bytes!("../assets/note_block.png");

// Resource to hold the song data
#[derive(Resource)]
pub struct SongData {
    pub notes: Vec<Note>,
}

#[derive(Component)]
pub struct NoteComponent {
    key: u8,       //  MIDI note number
    sound: u8,     // Index into the SOUNDS array
    velocity: f32, // Pixels per second
}

// Spawn notes based on the current tick
pub fn spawn_notes(
    mut commands: Commands,
    song: Res<SongData>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    app_state: Res<AppState>,
) {
    let current_tick = (time.elapsed_secs() * 1000.0) as u16; // Convert time to ticks
    let white_key_width = app_state.white_key_width;
    let key_spacing = app_state.key_spacing;

    for note in &song.notes {
        if note.tick <= current_tick {
            let _x_pos = (note.key as f32 - 60.0) * (white_key_width + key_spacing); // Adjust for middle C
            let _y_pos = 300.0; // Start at the top of the screen

            // Load the note texture
            let texture = asset_server.load("assets/note_block.png");

            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                NoteComponent {
                    key: note.key,
                    sound: note.instrument, // Use the instrument as the sound index
                    velocity: 100.0,        // Pixels per second
                },
            ));
        }
    }
}
