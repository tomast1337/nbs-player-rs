use bevy::prelude::*;
use nbs_rs::NbsParser;
use piano::setup_keyboard;
use std::str;

mod piano;

use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};

fn main() {
    let song_data = include_bytes!("../test-assets/nyan_cat.nbs");
    let mut song_file = NbsParser::new(song_data);
    let song = song_file.parse().unwrap();
    let notes = song.notes;

    for note in notes {
        println!("{:?}", note);
    }

    let mut app = App::new();
    app.add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_systems(Startup, setup_keyboard);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, toggle_wireframe);
    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}
