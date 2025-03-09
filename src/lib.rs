use bevy::{
    prelude::*,
    window::{self, PresentMode, WindowTheme},
};
use nbs_rs::{NbsFile, NbsParser};
use wasm_bindgen::prelude::*;

fn load_nbs_file(song_data: Option<Vec<u8>>) -> NbsFile {
    let song_data_bytes =
        song_data.unwrap_or_else(|| include_bytes!("../test-assets/nyan_cat.nbs").to_vec());
    let mut song_file = NbsParser::new(&song_data_bytes);
    let song = song_file.parse().unwrap();
    song
}

#[derive(Resource)]
pub struct SongData {
    pub nbs_file: NbsFile,
}

#[wasm_bindgen]
pub fn start(
    width: Option<f32>,
    height: Option<f32>,
    song_data: Option<Vec<u8>>,
    canvas_id: Option<String>,
) -> Result<(), JsValue> {
    // Load song
    let song = load_nbs_file(song_data);
    let mut app = App::new();

    app.add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (width.unwrap_or(1280.), height.unwrap_or(720.)).into(),
            canvas: Some(canvas_id.unwrap_or_else(|| "canvas".to_string())),
            ..default()
        }),
        exit_condition: bevy::window::ExitCondition::DontExit,
        close_when_requested: false,
        ..Default::default()
    }),))
        .insert_resource(SongData { nbs_file: song })
        .add_systems(Startup, (setup, setup_piano, setup_song_info));

    app.run();

    Ok(())
}

const X_EXTENT: f32 = 900.;

fn setup_song_info(mut commands: Commands, song: ResMut<SongData>) {
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let shapes = [
        meshes.add(Circle::new(50.0)),
        meshes.add(CircularSector::new(50.0, 1.0)),
        meshes.add(CircularSegment::new(50.0, 1.25)),
        meshes.add(Ellipse::new(25.0, 50.0)),
        meshes.add(Annulus::new(25.0, 50.0)),
        meshes.add(Capsule2d::new(25.0, 50.0)),
        meshes.add(Rhombus::new(75.0, 100.0)),
        meshes.add(Rectangle::new(50.0, 100.0)),
        meshes.add(RegularPolygon::new(50.0, 6)),
        meshes.add(Triangle2d::new(
            Vec2::Y * 50.0,
            Vec2::new(-50.0, -50.0),
            Vec2::new(50.0, -50.0),
        )),
    ];
    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                0.0,
                0.0,
            ),
        ));
    }
}

#[derive(Component)]
struct PianoKey {
    key: u8,
    label: String,
    is_pressed: bool,
    white_key_index: Option<usize>,
}

fn generate_piano_keys() -> (Vec<PianoKey>, Vec<PianoKey>) {
    let white_keys: [(&str, i32); 52] = [
        ("A0", 21),
        ("B0", 23),
        ("C1", 24),
        ("D1", 26),
        ("E1", 28),
        ("F1", 29),
        ("G1", 31),
        ("A1", 33),
        ("B1", 35),
        ("C2", 36),
        ("D2", 38),
        ("E2", 40),
        ("F2", 41),
        ("G2", 43),
        ("A2", 45),
        ("B2", 47),
        ("C3", 48),
        ("D3", 50),
        ("E3", 52),
        ("F3", 53),
        ("G3", 55),
        ("A3", 57),
        ("B3", 59),
        ("C4", 60),
        ("D4", 62),
        ("E4", 64),
        ("F4", 65),
        ("G4", 67),
        ("A4", 69),
        ("B4", 71),
        ("C5", 72),
        ("D5", 74),
        ("E5", 76),
        ("F5", 77),
        ("G5", 79),
        ("A5", 81),
        ("B5", 83),
        ("C6", 84),
        ("D6", 86),
        ("E6", 88),
        ("F6", 89),
        ("G6", 91),
        ("A6", 93),
        ("B6", 95),
        ("C7", 96),
        ("D7", 98),
        ("E7", 100),
        ("F7", 101),
        ("G7", 103),
        ("A7", 105),
        ("B7", 107),
        ("C8", 108),
    ];

    let black_keys: [(&str, i32); 36] = [
        ("A#0", 22),
        ("C#1", 25),
        ("D#1", 27),
        ("F#1", 30),
        ("G#1", 32),
        ("A#1", 34),
        ("C#2", 37),
        ("D#2", 39),
        ("F#2", 42),
        ("G#2", 44),
        ("A#2", 46),
        ("C#3", 49),
        ("D#3", 51),
        ("F#3", 54),
        ("G#3", 56),
        ("A#3", 58),
        ("C#4", 61),
        ("D#4", 63),
        ("F#4", 66),
        ("G#4", 68),
        ("A#4", 70),
        ("C#5", 73),
        ("D#5", 75),
        ("F#5", 78),
        ("G#5", 80),
        ("A#5", 82),
        ("C#6", 85),
        ("D#6", 87),
        ("F#6", 90),
        ("G#6", 92),
        ("A#6", 94),
        ("C#7", 97),
        ("D#7", 99),
        ("F#7", 102),
        ("G#7", 104),
        ("A#7", 106),
    ];

    let white_keys_vec: Vec<PianoKey> = white_keys
        .iter()
        .enumerate()
        .map(|(index, (label, key))| PianoKey {
            key: *key as u8,
            label: label.to_string(),
            is_pressed: false,
            white_key_index: Some(index),
        })
        .collect();

    let black_keys_vec: Vec<PianoKey> = black_keys
        .iter()
        .map(|(label, key)| {
            // Find the closest white key index for correct positioning
            let white_key_index = white_keys_vec
                .iter()
                .position(|white_key| white_key.key > *key as u8)
                .map(|index| index.saturating_sub(1)); // Get the preceding white key

            PianoKey {
                key: *key as u8,
                label: label.to_string(),
                is_pressed: false,
                white_key_index,
            }
        })
        .collect();

    (white_keys_vec, black_keys_vec)
}
fn setup_piano(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&mut Window>,
) {
    let (white_keys, black_keys) = generate_piano_keys();

    let window = window.single();
    let window_width = window.width();
    let window_height = window.height();

    let key_size_relative_to_screen = 0.1;
    let black_key_width_ratio = 1.;
    let black_key_height_ratio = 0.6;

    let key_spacing = 1.; // Spacing between keys

    let num_white_keys = white_keys.len() as f32;
    let white_key_width = (window_width / num_white_keys) - key_spacing;
    let white_key_height = window_height * key_size_relative_to_screen;
    let black_key_width = (white_key_width * black_key_width_ratio) - key_spacing;
    let black_key_height = white_key_height * black_key_height_ratio;

    let white_key_mesh = meshes.add(Rectangle::new(white_key_width, white_key_height));
    let black_key_mesh = meshes.add(Rectangle::new(black_key_width, black_key_height));
    let white_key_material = materials.add(Color::WHITE);
    let black_key_material = materials.add(Color::BLACK);

    // Draw white keys
    for (i, piano_key) in white_keys.iter().enumerate() {
        let key = piano_key.key;
        let x_pos =
            i as f32 * (white_key_width + key_spacing) - window_width / 2.0 + white_key_width / 2.0; // Centered on screen

        let y_pos = -window_height / 2.0 + white_key_height / 2.0;

        commands.spawn((
            Mesh2d(white_key_mesh.clone()),
            MeshMaterial2d(white_key_material.clone()),
            Transform::from_xyz(x_pos, y_pos, 0.0),
            PianoKey {
                is_pressed: false,
                key: key,
                label: piano_key.label.clone(),
                white_key_index: Some(i),
            },
        ));

        let label_x = x_pos;
        let label_y = -window_height / 2.0 + white_key_height + 10.;

        // Add label
        commands.spawn((
            Text2d::new(piano_key.label.to_string()),
            TextColor(Color::WHITE),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            Transform::from_xyz(label_x, label_y, 1.1),
        ));
    }
    // Draw black keys
    for (i, piano_key) in black_keys.iter().enumerate() {
        let key = piano_key.key;
        let white_key_index = piano_key.white_key_index.unwrap(); // The index of the corresponding white key

        let x_pos = (white_key_index as f32 + 0.5) * (white_key_width + key_spacing)
            - window_width / 2.0
            + (black_key_width / 2.0);

        let y_pos = -window_height / 2.0 + black_key_height / 2.0;

        commands.spawn((
            Mesh2d(black_key_mesh.clone()),
            MeshMaterial2d(black_key_material.clone()),
            Transform::from_xyz(x_pos, y_pos, 0.0),
            PianoKey {
                is_pressed: false,
                key: key,
                label: piano_key.label.clone(),
                white_key_index: Some(white_key_index),
            },
        ));
    }
}
