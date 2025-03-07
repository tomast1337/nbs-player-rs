use bevy::prelude::*;
use nbs_rs::NbsParser;
use notes::{spawn_notes, SongData};
use piano::setup_piano;
use std::{fs, str};

mod notes;
mod piano;
#[derive(Resource)]
pub struct SoundResources {
    sound: Vec<Handle<AudioSource>>,
}

fn setup_audio_files(asset_server: Res<AssetServer>, mut commands: Commands) {
    let sound_files = [
        "harp.ogg",           // 0 - Piano
        "bass.ogg",           // 1 - Double Bass
        "bd.ogg",             // 2 - Bass Drum
        "snare.ogg",          // 3 - Snare Drum
        "hat.ogg",            // 4 - Click
        "guitar.ogg",         // 5 - Guitar
        "flute.ogg",          // 6 - Flute
        "bell.ogg",           // 7 - Bell
        "icechime.ogg",       // 8 - Chime
        "xylobone.ogg",       // 9 - Xylophone
        "iron_xylophone.ogg", // 10 - Iron Xylophone
        "cow_bell.ogg",       // 11 - Cow Bell
        "didgeridoo.ogg",     // 12 - Didgeridoo
        "bit.ogg",            // 13 - Bit
        "banjo.ogg",          // 14 - Banjo
        "pling.ogg",          // 15 - Pling
    ];

    let mut sound_resource = SoundResources { sound: Vec::new() };

    for sound in sound_files.iter() {
        let sound_handle = asset_server.load(*sound);
        sound_resource.sound.push(sound_handle);
    }

    commands.insert_resource(sound_resource);
}

#[derive(Resource)]
pub struct AppState {
    pub current_tick: u16,
    pub white_key_width: f32,
    pub white_key_height: f32,
    pub black_key_width: f32,
    pub black_key_height: f32,
    pub key_spacing: f32,
    pub black_keys: Vec<(String, i32)>,
    pub white_keys: Vec<(String, i32)>,
    pub window_width: f32,
    pub window_height: f32,
}

fn setup(mut commands: Commands, windows: Query<&mut Window>) {
    let white_keys: [(&str, i32); 52] = [
        ("A0", 21), ("B0", 23), ("C1", 24), ("D1", 26),
        ("E1", 28), ("F1", 29), ("G1", 31), ("A1", 33),
        ("B1", 35), ("C2", 36), ("D2", 38), ("E2", 40),
        ("F2", 41), ("G2", 43), ("A2", 45), ("B2", 47),
        ("C3", 48), ("D3", 50), ("E3", 52), ("F3", 53),
        ("G3", 55), ("A3", 57), ("B3", 59), ("C4", 60),
        ("D4", 62), ("E4", 64), ("F4", 65), ("G4", 67),
        ("A4", 69), ("B4", 71), ("C5", 72), ("D5", 74),
        ("E5", 76), ("F5", 77), ("G5", 79), ("A5", 81),
        ("B5", 83), ("C6", 84), ("D6", 86), ("E6", 88),
        ("F6", 89), ("G6", 91), ("A6", 93), ("B6", 95),
        ("C7", 96), ("D7", 98), ("E7", 100), ("F7", 101),
        ("G7", 103), ("A7", 105), ("B7", 107), ("C8", 108),
    ];

    let black_keys: [(&str, i32); 36] = [
        ("A#0", 1), ("C#1", 3),("D#1", 4), ("F#1", 6),
        ("G#1", 7), ("A#1", 8),("C#2", 10), ("D#2", 11),
        ("F#2", 13), ("G#2", 14),("A#2", 15), ("C#3", 17),
        ("D#3", 18), ("F#3", 20),("G#3", 21), ("A#3", 22),
        ("C#4", 24), ("D#4", 25),("F#4", 27), ("G#4", 28),
        ("A#4", 29), ("C#5", 31),("D#5", 32), ("F#5", 34),
        ("G#5", 35), ("A#5", 36),("C#6", 38), ("D#6", 39),
        ("F#6", 41), ("G#6", 42),("A#6", 43), ("C#7", 45),
        ("D#7", 46), ("F#7", 48),("G#7", 49), ("A#7", 50),
    ];

    let white_keys_vec: Vec<(String, i32)> = white_keys.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    let black_keys_vec: Vec<(String, i32)> = black_keys.iter().map(|(k, v)| (k.to_string(), *v)).collect();
   
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();

    let key_size_relative_to_screen = 0.1;
    let black_key_width_ratio = 0.6;
    let black_key_height_ratio = 0.6;

    let num_white_keys = white_keys.len() as f32;
    let white_key_width = window_width / num_white_keys;
    let white_key_height = window_height * key_size_relative_to_screen;
    let black_key_width = white_key_width * black_key_width_ratio;
    let black_key_height = white_key_height * black_key_height_ratio;

    let key_spacing = 0.01; // Spacing between keys

    commands.insert_resource(AppState {
        current_tick: 0,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
        key_spacing,
        black_keys:white_keys_vec,
        white_keys:black_keys_vec,
        window_width: window_width,
        window_height: window_height,
    });
}

fn main() {
    let song_data = fs::read("./test-assets/nyan_cat.nbs").expect("Failed to read the .nbs file");
    let mut song_file = NbsParser::new(song_data.as_slice());
    let song = song_file.parse().unwrap();
    let notes = song.notes;

    let song_name = str::from_utf8(&song.header.song_name).unwrap();
    let song_author = str::from_utf8(&song.header.song_author).unwrap();

    let title = format!("{} - {}", song_name, song_author);

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }))
    .insert_resource(SongData { notes })
    .add_systems(Startup, setup_piano)
    .add_systems(Startup, setup_audio_files)
    .add_systems(Startup, spawn_notes);

    app.run();
}
