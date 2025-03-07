use bevy::{prelude::*, window::WindowTheme};
use nbs_rs::{NbsParser, Note};
use piano::setup_piano;
use std::{fs, str};

mod piano;

#[derive(Resource)]
pub struct SoundResources {
    sound: Vec<Handle<AudioSource>>,
}

#[derive(Resource)]
pub struct SongData {
    sound: Vec<Note>,
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

fn main() {
    // print current working directory
    println!(
        "Current working directory: {:?}",
        std::env::current_dir().unwrap()
    );

    let song_data = fs::read("./test-assets/nyan_cat.nbs").expect("Failed to read the .nbs file");
    let mut song_file = NbsParser::new(song_data.as_slice());
    let song = song_file.parse().unwrap();
    let notes = song.notes;

    for note in notes {
        println!("{:?}", note);
    }

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
    .add_systems(Startup, setup_piano)
    .add_systems(Startup, setup_audio_files);

    app.run();
}
