use bevy::prelude::*;
use nbs_rs::Note;

use crate::piano::{self, PianoKey};

static NOTE_TEXTURE: &[u8] = include_bytes!("../assets/note_block.png");

// Resource to hold the song data
#[derive(Resource)]
pub struct Song {
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
    song: Res<Song>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    let current_tick = (time.elapsed_secs() * 1000.0) as u16; // Convert time to ticks

    for note in &song.notes {
        if note.tick <= current_tick {
            let _x_pos = (note.key as f32 - 60.0) * (piano::WHITE_KEY_WIDTH + piano::KEY_SPACING); // Adjust for middle C
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

// Move notes downward
pub fn move_notes(mut query: Query<(&mut Transform, &NoteComponent)>, time: Res<Time>) {
    for (mut transform, note) in &mut query {
        transform.translation.y -= note.velocity * time.delta_secs();
    }
}

// Play sounds when notes reach the piano roll
pub fn play_notes(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &NoteComponent)>,
    mut key_query: Query<&mut PianoKey>,
    audio: Res<Audio>,
    sounds: Res<Assets<AudioSource>>,
) {
    for (entity, transform, note) in &query {
        if transform.translation.y <= -250.0 {
            // Play the sound
            let sound = sounds.get(SOUNDS[note.sound as usize]).unwrap();
            audio.play(sound.clone());

            // Reset the key state after a short delay
            for mut piano_key in &mut key_query {
                if piano_key.key == note.key {
                    piano_key.is_pressed = true;
                }
            }

            // Despawn the note
            commands.entity(entity).despawn();
        }
    }
}
