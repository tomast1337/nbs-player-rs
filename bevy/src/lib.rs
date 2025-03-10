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

    app.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (width.unwrap_or(848.), height.unwrap_or(480.)).into(),
                canvas: Some(canvas_id.unwrap_or_else(|| "canvas".to_string())),
                ..default()
            }),
            exit_condition: bevy::window::ExitCondition::OnPrimaryClosed,
            close_when_requested: false,
            ..Default::default()
        })
        .set(AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
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

#[derive(Resource)]
struct NoteSounds {
    sounds: bevy::utils::hashbrown::HashMap<u8, Handle<AudioSource>>, // Maps note keys to audio handles
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.insert_resource(NoteSounds {
        sounds: bevy::utils::hashbrown::HashMap::from([
            (0, asset_server.load("bass.ogg")),
            (1, asset_server.load("bd.ogg")),
            (2, asset_server.load("harp.ogg")),
            (3, asset_server.load("snare.ogg")),
            (4, asset_server.load("hat.ogg")),
            (5, asset_server.load("guitar.ogg")),
            (6, asset_server.load("flute.ogg")),
            (7, asset_server.load("bell.ogg")),
            (8, asset_server.load("icechime.ogg")),
            (9, asset_server.load("xylobone.ogg")),
            (10, asset_server.load("iron_xylophone.ogg")),
            (11, asset_server.load("cow_bell.ogg")),
            (12, asset_server.load("didgeridoo.ogg")),
            (13, asset_server.load("bit.ogg")),
            (14, asset_server.load("banjo.ogg")),
            (15, asset_server.load("pling.ogg")),
        ]),
    });
}
