use glow::HasContext;
use log;
use nbs_rs::{NbsFile, NbsParser, Note};
use std::str;
use std::thread;
use std::time::{Duration, Instant};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window, AudioBuffer, AudioBufferSourceNode, AudioContext};

struct PianoKey {
    key: u8,
    label: String,
    is_pressed: bool,
}

pub struct App {
    canvas_width: f32,
    canvas_height: f32,
    // gl: glow::Context, // WebGL context
    black_keys: Vec<PianoKey>,
    white_keys: Vec<PianoKey>,
    song: NbsFile,

    audio_buffers: Vec<AudioBuffer>,
    audio_ctx: AudioContext,

    song_name: String,
    current_tick: u32,
    white_key_width: f32,
    white_key_height: f32,
    black_key_width: f32,
    black_key_height: f32,
    paused: bool,
}

impl App {
    pub fn run(&mut self) {
        log::info!("Running app");
        let update_interval = Duration::from_millis(50); // 20 ticks per second (1000ms / 20 = 50ms)
        let mut next_update = Instant::now() + update_interval;

        loop {
            let now = Instant::now();

            // Update at 20 ticks per second
            if !self.paused && now >= next_update {
                self.update();
                next_update = now + update_interval;
            }

            // Render at 120 FPS (or as fast as possible)
            self.render();

            // Yield control to the OS to avoid busy-waiting
            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn update(&mut self) {
        log::info!("Updating app");
        // Update game logic here
        self.current_tick += 1;
    }

    pub fn render(&self) {
        log::info!("Rendering app");
    }

    pub fn change_tick(&mut self, tick: u32) {
        self.current_tick = tick;
    }

    pub fn play_sound(&self, index: usize) -> Result<(), JsValue> {
        if index >= self.audio_buffers.len() {
            return Err(JsValue::from_str("Index out of bounds"));
        }

        // Ensure the audio context is running
        if self.audio_ctx.state() == web_sys::AudioContextState::Suspended {
            let promise = self.audio_ctx.resume();
            match promise {
                Ok(promise) => {
                    let future = JsFuture::from(promise);
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Err(e) = future.await {
                            web_sys::console::error_1(&e);
                        }
                    });
                }
                Err(e) => {
                    web_sys::console::error_1(&e);
                }
            }
        }

        // Create a new AudioBufferSourceNode
        let source = self.audio_ctx.create_buffer_source()?;
        source.set_buffer(Some(&self.audio_buffers[index]));

        // Connect the source to the destination (speakers)
        source.connect_with_audio_node(&self.audio_ctx.destination())?;

        // Start playback
        source.start()?;

        Ok(())
    }
}

async fn load_audio() -> Result<Vec<AudioBuffer>, JsValue> {
    let bass = include_bytes!("../assets/bass.ogg"); // 1 - Double Bass
    let bd = include_bytes!("../assets/bd.ogg"); // 2 - Bass Drum
    let harp = include_bytes!("../assets/harp.ogg"); // 0 - Piano
    let snare = include_bytes!("../assets/snare.ogg"); // 3 - Snare Drum
    let hat = include_bytes!("../assets/hat.ogg"); // 4 - Click
    let guitar = include_bytes!("../assets/guitar.ogg"); // 5 - Guitar
    let flute = include_bytes!("../assets/flute.ogg"); // 6 - Flute
    let bell = include_bytes!("../assets/bell.ogg"); // 7 - Bell
    let icechime = include_bytes!("../assets/icechime.ogg"); // 8 - Chime
    let xylobone = include_bytes!("../assets/xylobone.ogg"); // 9 - Xylophone
    let iron_xylophone = include_bytes!("../assets/iron_xylophone.ogg"); // 10 - Iron Xylophone
    let cow_bell = include_bytes!("../assets/cow_bell.ogg"); // 11 - Cow Bell
    let didgeridoo = include_bytes!("../assets/didgeridoo.ogg"); // 12 - Didgeridoo
    let bit = include_bytes!("../assets/bit.ogg"); // 13 - Bit
    let banjo = include_bytes!("../assets/banjo.ogg"); // 14 - Banjo
    let pling = include_bytes!("../assets/pling.ogg"); // 15 - Pling

    let sounds: Vec<&[u8]> = vec![
        bass,
        bd,
        harp,
        snare,
        hat,
        guitar,
        flute,
        bell,
        icechime,
        xylobone,
        iron_xylophone,
        cow_bell,
        didgeridoo,
        bit,
        banjo,
        pling,
    ];

    // Create an AudioContext
    let audio_ctx = AudioContext::new().unwrap();

    // Decode each sound and store the resulting AudioBuffer
    let mut audio_buffers = Vec::new();
    for sound in sounds {
        let array_buffer = js_sys::Uint8Array::from(sound).buffer();
        let audio_buffer_promise = audio_ctx.decode_audio_data(&array_buffer)?;
        let audio_buffer = wasm_bindgen_futures::JsFuture::from(audio_buffer_promise).await?;
        let audio_buffer: AudioBuffer = audio_buffer.dyn_into().unwrap();
        audio_buffers.push(audio_buffer);
    }

    // Return the array of AudioBuffers
    Ok(audio_buffers)
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

    let white_keys_vec: Vec<PianoKey> = white_keys
        .iter()
        .map(|(label, key)| PianoKey {
            key: *key as u8,
            label: label.to_string(),
            is_pressed: false,
        })
        .collect();
    let black_keys_vec: Vec<PianoKey> = black_keys
        .iter()
        .map(|(label, key)| PianoKey {
            key: *key as u8,
            label: label.to_string(),
            is_pressed: false,
        })
        .collect();
    (white_keys_vec, black_keys_vec)
}

fn calculate_key_dimensions(
    canvas_width: f32,
    canvas_height: f32,
    white_keys_vec: &Vec<PianoKey>,
) -> (f32, f32, f32, f32) {
    // Calculate key sizes
    let key_size_relative_to_screen = 0.1;
    let black_key_width_ratio = 1.0;
    let black_key_height_ratio = 0.6;

    let num_white_keys = white_keys_vec.len() as f32;
    let white_key_width = canvas_width / num_white_keys;
    let white_key_height = canvas_height * key_size_relative_to_screen;
    let black_key_width = white_key_width * black_key_width_ratio;
    let black_key_height = white_key_height * black_key_height_ratio;
    (
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
    )
}

async fn setup(
    width: Option<f32>,
    height: Option<f32>,
    song_data: Option<&[u8]>,
) -> Result<App, JsValue> {
    // 16:9 aspect ratio
    let canvas_width = width.unwrap_or(1280.0);
    let canvas_height = height.unwrap_or(720.0);

    let document = window().unwrap().document().unwrap();

    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let webgl2_context = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()?;

    let gl: glow::Context;

    let (white_keys_vec, black_keys_vec) = generate_piano_keys();

    // Load song
    let song_data_bytes = song_data.unwrap_or(include_bytes!("../test-assets/nyan_cat.nbs"));
    let mut song_file = NbsParser::new(song_data_bytes);
    let song = song_file.parse().unwrap();
    let song_name = str::from_utf8(&song.header.song_name).unwrap();
    let song_author = str::from_utf8(&song.header.song_author).unwrap();

    let title = format!("{} - {}", song_name, song_author);

    log::info!("Loaded song: {}", title);

    let (white_key_width, white_key_height, black_key_width, black_key_height) =
        calculate_key_dimensions(canvas_width, canvas_height, &white_keys_vec);

    // Load audio
    let audio_buffers = load_audio().await?;

    let app = App {
        paused: true,
        canvas_width,
        canvas_height,
        //gl,
        audio_buffers,
        audio_ctx: AudioContext::new().unwrap(),
        black_keys: black_keys_vec,
        white_keys: white_keys_vec,
        song: song,
        song_name: title,
        current_tick: 0,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
    };

    Ok(app)
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    console_error_panic_hook::set_once();

    spawn_local(async {
        match setup(None, None, None).await {
            Ok(app) => {
                log::info!("Setup successful!");
                app.play_sound(3).unwrap();
            }
            Err(e) => {
                log::info!("Error: {:?}", e);
            }
        }
    });

    Ok(())
}
