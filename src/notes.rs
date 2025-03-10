use bevy::prelude::*;

use crate::{piano, song::SongData, NoteSounds};
use nbs_rs;
#[derive(Component)]
pub struct Note {
    speed: f32, // Speed at which the note falls
    note: nbs_rs::Note,
    was_played: bool,
}

pub fn setup_notes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    song: Res<SongData>,
    window: Query<&Window>,
) {
    // TODO: This part is not working correctly
    let window = window.single();
    let window_width = window.width();
    let window_height = window.height();

    let song = &song.nbs_file;
    let notes = &song.notes;

    let key_spacing = 1.; // Spacing between keys

    let num_white_keys = 52;
    let white_key_width = (window_width / num_white_keys as f32) - key_spacing;

    let white_keys = piano::generate_piano_keys().0;

    let note_texture_handle: Handle<Image> = asset_server.load("note_block.png");

    let layers_len = song.layers.len() as f32;

    let note_mesh = meshes.add(Rectangle::new(white_key_width, white_key_width));

    for note in notes {
        let key = note.key;
        if let Some(key_index) = white_keys.iter().position(|white_key| white_key.key == key) {
            let x_pos = key_index as f32 * (white_key_width + key_spacing) - window_width / 2.0
                + white_key_width / 2.0; // Centered on screen
            let y_pos = window_height + white_key_width * note.tick as f32;

            let note_layer = note.layer as f32;

            let note_material = materials.add(ColorMaterial {
                alpha_mode: bevy::sprite::AlphaMode2d::Opaque,
                texture: Some(note_texture_handle.clone()),
                color: Color::hsl(note_layer / layers_len, 1.0, 0.5),
            });

            commands.spawn((
                Mesh2d(note_mesh.clone()),
                MeshMaterial2d(note_material.clone()),
                Transform::from_xyz(x_pos, y_pos, -1.0),
                Note {
                    speed: 100.0,
                    note: note.clone(),
                    was_played: false,
                },
            ));
        } else {
            // Handle the case where the key is not found
            eprintln!("Key not found: {}", key);
        }
    }
}

pub fn update_notes(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Note)>,
    window: Query<&mut Window>,
    piano_data: Res<piano::PianoData>,
    note_sounds: Res<NoteSounds>,
    mut commands: Commands,
) {
    let window = window.single();
    let window_height = window.height();
    let white_key_height = piano_data.white_key_height;
    let y_target = -window_height / 2.0 - white_key_height;
    for (mut transform, mut note) in query.iter_mut() {
        transform.translation.y -= note.speed * time.delta_secs();

        if transform.translation.y < y_target && !note.was_played {
            //let note_key = note.note.key;
            let note_instrument = note.note.instrument;
            let note_sound = note_sounds.sounds.get(&note_instrument).unwrap();
            let audio = note_sound.clone();
            commands.spawn(AudioPlayer::new(audio));
            note.was_played = true;
        }
    }
}
