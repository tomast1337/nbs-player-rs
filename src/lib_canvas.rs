use log;
use nbs_rs::{NbsFile, NbsParser};
use std::cell::RefCell;
use std::rc::Rc;
use std::str;
use std::thread;
use std::time::{Duration, Instant};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, AudioBuffer, AudioContext};

async fn load_audio(audio_ctx: &AudioContext) -> Result<Vec<AudioBuffer>, JsValue> {
    let sounds: Vec<&[u8]> = vec![
        include_bytes!("../assets/bass.ogg"),
        include_bytes!("../assets/bd.ogg"),
        include_bytes!("../assets/harp.ogg"),
        include_bytes!("../assets/snare.ogg"),
        include_bytes!("../assets/hat.ogg"),
        include_bytes!("../assets/guitar.ogg"),
        include_bytes!("../assets/flute.ogg"),
        include_bytes!("../assets/bell.ogg"),
        include_bytes!("../assets/icechime.ogg"),
        include_bytes!("../assets/xylobone.ogg"),
        include_bytes!("../assets/iron_xylophone.ogg"),
        include_bytes!("../assets/cow_bell.ogg"),
        include_bytes!("../assets/didgeridoo.ogg"),
        include_bytes!("../assets/bit.ogg"),
        include_bytes!("../assets/banjo.ogg"),
        include_bytes!("../assets/pling.ogg"),
    ];

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

fn load_nbs_file(song_data: Option<&[u8]>) -> NbsFile {
    let song_data_bytes = song_data.unwrap_or(include_bytes!("../test-assets/nyan_cat.nbs"));
    let mut song_file = NbsParser::new(song_data_bytes);
    let song = song_file.parse().unwrap();
    song
}

struct PianoKey {
    key: u8,
    label: String,
    is_pressed: bool,
}

fn now() -> f64 {
    if let Some(window) = window() {
        if let Some(performance) = window.performance() {
            return performance.now();
        }
    }
    panic!("Unable to get current time");
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    if let Some(window) = window() {
        window
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("Failed to request animation frame");
    }
}

pub struct App {
    canvas_width: f32,
    canvas_height: f32,
    ctx: web_sys::CanvasRenderingContext2d,
    black_keys: Vec<PianoKey>,
    white_keys: Vec<PianoKey>,
    white_key_width: f32,
    white_key_height: f32,
    black_key_width: f32,
    black_key_height: f32,

    song: NbsFile,

    audio_buffers: Vec<AudioBuffer>,
    audio_ctx: AudioContext,

    current_tick: u32,
    paused: bool,
}

impl App {
    pub async fn new(
        width: Option<f32>,
        height: Option<f32>,
        song_data: Option<&[u8]>,
        canvas_id: Option<&str>,
    ) -> Result<Rc<RefCell<App>>, JsValue> {
        // Load song
        let song = load_nbs_file(song_data);

        // Load audio
        let audio_ctx = AudioContext::new().unwrap();
        let audio_buffers = load_audio(&audio_ctx).await?;

        // 16:9 aspect ratio
        let canvas_width = width.unwrap_or(1280.0);
        let canvas_height = height.unwrap_or(720.0);

        let (white_keys, black_keys) = generate_piano_keys();

        let (white_key_width, white_key_height, black_key_width, black_key_height) =
            calculate_key_dimensions(canvas_width, canvas_height, &white_keys);

        let document = window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id.unwrap_or("canvas").as_ref())
            .ok_or_else(|| JsValue::from_str("Canvas element not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        // Set the canvas size
        canvas.set_width(canvas_width as u32);
        canvas.set_height(canvas_height as u32);

        // Initialize canvas context
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

        Ok(Rc::new(RefCell::new(App {
            paused: true,
            canvas_width,
            canvas_height,
            ctx,
            audio_buffers,
            audio_ctx,
            black_keys,
            white_keys,
            song: song,
            current_tick: 0,
            white_key_width,
            white_key_height,
            black_key_width,
            black_key_height,
        })))
    }

    pub fn run(&self) {}

    pub fn update(&mut self) {
        // Update game logic here
        self.current_tick += 1;
    }

    pub fn render(&self) {
        // Clear the canvas
        self.ctx.clear_rect(
            0.0,
            0.0,
            self.canvas_width as f64,
            self.canvas_height as f64,
        );

        // draw song info
        self.draw_song_info();

        // draw piano keys
        //self.draw_piano_keys();

        // draw note
        //self.draw_notes();
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

    fn draw_piano_keys(&self) {
        todo!()
    }

    fn draw_song_info(&self) {
        let song = &self.song;
        let song_name = str::from_utf8(&song.header.song_name).unwrap();
        let song_author = str::from_utf8(&song.header.song_author).unwrap();
        let song_tempo = song.header.tempo;
        let song_ticks = song.header.song_length;
        let current_tick = self.current_tick;

        self.ctx.set_font("20px Arial");
        self.ctx.set_fill_style(&JsValue::from_str("black"));
        self.ctx
            .fill_text(&format!("Name: {}", song_name), 10.0, 20.0)
            .unwrap();

        self.ctx
            .fill_text(&format!("Author: {}", song_author), 10.0, 40.0)
            .unwrap();

        self.ctx
            .fill_text(&format!("Tempo: {}", song_tempo), 10.0, 60.0)
            .unwrap();

        self.ctx
            .fill_text(&format!("Ticks: {}", song_ticks), 10.0, 80.0)
            .unwrap();

        self.ctx
            .fill_text(&format!("Current Tick: {}", current_tick), 10.0, 100.0)
            .unwrap();

        self.ctx
            .fill_text(
                &format!("{} of {} ticks", current_tick, song_ticks),
                10.0,
                120.0,
            )
            .unwrap();
    }

    fn draw_notes(&self) {
        todo!()
    }
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    console_error_panic_hook::set_once();

    spawn_local(async {
        match App::new(None, None, None, None).await {
            Ok(app) => {
                log::info!("Setup successful!");
                app.borrow_mut().run();
            }
            Err(e) => {
                log::info!("Error: {:?}", e);
            }
        }
    });

    Ok(())
}
