use bevy::prelude::*;
use nbs_rs::NbsParser;
use std::str;

static SOUNDS: &[&[u8]] = &[
    include_bytes!("../assets/harp.ogg"),           // 0 - Piano
    include_bytes!("../assets/bass.ogg"),           // 1 - Double Bass
    include_bytes!("../assets/bd.ogg"),             // 2 - Bass Drum
    include_bytes!("../assets/snare.ogg"),          // 3 - Snare Drum
    include_bytes!("../assets/hat.ogg"),            // 4 - Click
    include_bytes!("../assets/guitar.ogg"),         // 5 - Guitar
    include_bytes!("../assets/flute.ogg"),          // 6 - Flute
    include_bytes!("../assets/bell.ogg"),           // 7 - Bell
    include_bytes!("../assets/icechime.ogg"),       // 8 - Chime
    include_bytes!("../assets/xylobone.ogg"),       // 9 - Xylophone
    include_bytes!("../assets/iron_xylophone.ogg"), // 10 - Iron Xylophone
    include_bytes!("../assets/cow_bell.ogg"),       // 11 - Cow Bell
    include_bytes!("../assets/didgeridoo.ogg"),     // 12 - Didgeridoo
    include_bytes!("../assets/bit.ogg"),            // 13 - Bit
    include_bytes!("../assets/banjo.ogg"),          // 14 - Banjo
    include_bytes!("../assets/pling.ogg"),          // 15 - Pling
];

use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        #[cfg(not(target_arch = "wasm32"))]
        Wireframe2dPlugin,
    ))
    .add_systems(Startup, setup);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, toggle_wireframe);
    app.run();
}

const WHITE_KEY_WIDTH: f32 = 40.0;
const WHITE_KEY_HEIGHT: f32 = 160.0;
const BLACK_KEY_WIDTH: f32 = 25.0;
const BLACK_KEY_HEIGHT: f32 = 100.0;
const KEY_SPACING: f32 = 2.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let white_keys = [
        "A0", "B0", "C1", "D1", "E1", "F1", "G1", "A1", "B1", "C2", "D2", "E2", "F2", "G2", "A2",
        "B2", "C3", "D3", "E3", "F3", "G3", "A3", "B3", "C4", "D4", "E4", "F4", "G4", "A4", "B4",
        "C5", "D5", "E5", "F5", "G5", "A5", "B5", "C6", "D6", "E6", "F6", "G6", "A6", "B6", "C7",
        "D7", "E7", "F7", "G7", "A7", "B7", "C8",
    ];

    let black_keys = [
        ("A#0", 1),
        ("C#1", 3),
        ("D#1", 4),
        ("F#1", 6),
        ("G#1", 7),
        ("A#1", 8),
        ("C#2", 10),
        ("D#2", 11),
        ("F#2", 13),
        ("G#2", 14),
        ("A#2", 15),
        ("C#3", 17),
        ("D#3", 18),
        ("F#3", 20),
        ("G#3", 21),
        ("A#3", 22),
        ("C#4", 24),
        ("D#4", 25),
        ("F#4", 27),
        ("G#4", 28),
        ("A#4", 29),
        ("C#5", 31),
        ("D#5", 32),
        ("F#5", 34),
        ("G#5", 35),
        ("A#5", 36),
        ("C#6", 38),
        ("D#6", 39),
        ("F#6", 41),
        ("G#6", 42),
        ("A#6", 43),
        ("C#7", 45),
        ("D#7", 46),
        ("F#7", 48),
        ("G#7", 49),
        ("A#7", 50),
    ];

    // Draw white keys
    for (i, key) in white_keys.iter().enumerate() {
        let x_pos = i as f32 * (WHITE_KEY_WIDTH + KEY_SPACING) - 450.0; // Centered on screen

        // Spawn white key
        commands.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(WHITE_KEY_WIDTH, WHITE_KEY_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(x_pos, -250.0, 0.0), // Bottom of screen
            GlobalTransform::default(),
            Visibility::default(),
        ));

        // Add label
        commands.spawn((
            Text2d::new(key.to_string()),
            TextColor(Color::BLACK),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            Transform::from_xyz(x_pos, -200.0, 1.1),
            GlobalTransform::default(),
            Visibility::default(),
        ));
    }

    // Draw black keys
    for (key, index) in black_keys.iter() {
        let x_pos =
            *index as f32 * (WHITE_KEY_WIDTH + KEY_SPACING) - 450.0 - (BLACK_KEY_WIDTH / 2.0);

        // Spawn black key
        commands.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(BLACK_KEY_WIDTH, BLACK_KEY_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(x_pos, -200.0, 1.0), // Higher z-index
            GlobalTransform::default(),
            Visibility::default(),
        ));

        // Add label
        commands.spawn((
            Text2d::new(key.to_string()),
            TextColor(Color::WHITE),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            Transform::from_xyz(x_pos, -200.0, 1.1),
            GlobalTransform::default(),
            Visibility::default(),
        ));
    }
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
