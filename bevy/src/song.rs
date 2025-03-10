use bevy::prelude::*;
use nbs_rs::{NbsFile, NbsParser};

#[derive(Resource)]
pub struct SongData {
    pub nbs_file: NbsFile,
}

pub fn load_nbs_file(song_data: Option<Vec<u8>>) -> NbsFile {
    let song_data_bytes =
        song_data.unwrap_or_else(|| include_bytes!("../test-assets/nyan_cat.nbs").to_vec());
    let mut song_file = NbsParser::new(&song_data_bytes);
    let song = song_file.parse().unwrap();
    song
}

pub fn setup_song_info(mut commands: Commands, song: ResMut<SongData>) {
    let song = &song.nbs_file;
    let original_author =
        String::from_utf8(song.header.original_author.clone()).unwrap_or_default();
    let song_name = String::from_utf8(song.header.song_name.clone()).unwrap_or_default();
    //let tempo = song.header.tempo;
    //let length = song.header.song_length;

    let title = format!("{} - {}", song_name, original_author);

    commands.spawn((
        Text2d::new(title),
        TextColor(Color::BLACK),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Transform::from_xyz(0., 0., 1.1),
    ));
}
