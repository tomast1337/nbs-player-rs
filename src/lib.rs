use bevy::prelude::*;
use song::SongData;
use wasm_bindgen::prelude::*;

mod notes;
mod piano;
mod song;

#[wasm_bindgen]
pub fn start(
    width: Option<f32>,
    height: Option<f32>,
    song_data: Option<Vec<u8>>,
    canvas_id: Option<String>,
) -> Result<(), JsValue> {
    // Load song
    let song = song::load_nbs_file(song_data);
    let mut app = App::new();

    app.add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (width.unwrap_or(848.), height.unwrap_or(480.)).into(),
            canvas: Some(canvas_id.unwrap_or_else(|| "canvas".to_string())),
            ..default()
        }),
        exit_condition: bevy::window::ExitCondition::DontExit,
        close_when_requested: false,
        ..Default::default()
    }),))
        .insert_resource(SongData { nbs_file: song })
        .add_systems(
            Startup,
            (
                setup,
                piano::setup_piano,
                song::setup_song_info,
                notes::setup_notes,
            ),
        )
        .add_systems(Update, notes::update_notes);

    app.run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
}
