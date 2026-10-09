//! Canvas 2D frontend for the NBS player. Web only: it needs no GPU API, so it runs
//! anywhere a `<canvas>` does. Backgrounds are a CPU approximation of the shader ones.
//!
//! Build with `build-web.sh`, then call the exported `start(canvas_id, config_json, song_url)`
//! (see `web/index.html`).

#![cfg(target_arch = "wasm32")]

mod canvas;

use std::cell::RefCell;
use std::rc::Rc;

use nbs_player_core::app::App;
use nbs_player_core::audio::{AudioBackend, InstrumentBank, NullBackend};
use nbs_player_core::config::AppConfig;
use nbs_player_core::player::Player;
use nbs_player_core::render::InputState;
use nbs_player_core::theme::Theme;
use nbs_player_core::types::Vec2;
use nbs_player_core::{notes, profiler, song};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent};

use canvas::Canvas2d;

struct State {
    canvas: HtmlCanvasElement,
    gfx: Canvas2d,
    app: App,
    bank: InstrumentBank,
    audio: Box<dyn AudioBackend>,
    audio_ready: bool,
    input: InputState,
    last_ms: f64,
}

impl State {
    /// Open the audio device (once) and hand it the instruments. Must run inside a user
    /// gesture handler, as browsers only start audio from one.
    fn ensure_audio(&mut self) {
        if self.audio_ready {
            return;
        }
        self.audio_ready = true;
        let mut audio: Box<dyn AudioBackend> = match nbs_player_cpal::CpalBackend::new() {
            Ok(b) => Box::new(b),
            Err(e) => {
                log::error!("audio unavailable ({e}); running silent");
                Box::new(NullBackend)
            }
        };
        self.bank.install(audio.as_mut());
        audio.set_master_volume(self.app.volume);
        self.audio = audio;
    }

    fn frame(&mut self, now_ms: f64) {
        let dt = ((now_ms - self.last_ms) / 1000.0) as f32;
        self.last_ms = now_ms;

        let size = self.gfx.begin_frame(&self.canvas);
        let events = self
            .app
            .frame(&self.input, dt, size, &mut self.gfx, self.audio.as_mut());
        // edge-triggered inputs last one frame
        self.input.mouse_pressed = false;
        self.input.mouse_released = false;
        self.input.space_pressed = false;
        self.input.toggle_profiler = false;
        self.input.dump_profile = false;

        if events.toggle_fullscreen {
            let document = web_sys::window().and_then(|w| w.document());
            if let Some(document) = document {
                if document.fullscreen_element().is_some() {
                    document.exit_fullscreen();
                } else if let Err(e) = self.canvas.request_fullscreen() {
                    log::warn!("fullscreen refused: {e:?}");
                }
            }
        }
    }
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}

/// Parse the song and set up the player, before any canvas or audio exists.
fn prepare(song_bytes: Option<&[u8]>) -> Result<(Player, InstrumentBank), String> {
    let song_data = song::load_nbs_file(song_bytes).map_err(|e| format!("loading song: {e}"))?;
    let header = &song_data.song.header;

    let bank = InstrumentBank::new(&song_data.extra_sounds);
    let note_blocks = notes::get_note_blocks(&song_data.song, &bank.base_keys());
    let name = String::from_utf8(header.song_name.clone()).unwrap_or_else(|_| "Unknown".into());
    let author = String::from_utf8(header.song_author.clone()).unwrap_or_else(|_| "Unknown".into());
    let player = Player::new(
        format!("{name} - {author}"),
        header.tempo,
        header.song_length as usize,
        note_blocks,
        notes::generate_instrument_palette(),
    );
    Ok((player, bank))
}

async fn fetch_bytes(url: &str) -> Option<Vec<u8>> {
    let window = web_sys::window()?;
    let response = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url))
        .await
        .ok()?;
    let response: web_sys::Response = response.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let buffer = wasm_bindgen_futures::JsFuture::from(response.array_buffer().ok()?)
        .await
        .ok()?;
    Some(js_sys::Uint8Array::new(&buffer).to_vec())
}

fn listen<E: JsCast + 'static>(
    target: &web_sys::EventTarget,
    event: &str,
    state: &Rc<RefCell<State>>,
    handler: impl Fn(&mut State, E) + 'static,
) {
    let state = state.clone();
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |e: JsValue| {
        if let Ok(e) = e.dyn_into::<E>() {
            handler(&mut state.borrow_mut(), e);
        }
    });
    let _ = target.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
    closure.forget();
}

/// Start the player on the `<canvas id="canvas_id">`.
/// `config_json` is an `AppConfig` (empty = demo theme); `song_url` is fetched as a `.nbs`
/// or zip (empty or failing = the bundled demo song).
#[wasm_bindgen]
pub async fn start(canvas_id: String, config_json: String, song_url: String) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
    // std's clock panics on wasm32-unknown-unknown.
    profiler::set_clock(|| (now_ms() * 1e6) as u64);

    let err = |e: String| JsValue::from_str(&e);
    let config: AppConfig = if config_json.trim().is_empty() {
        AppConfig::demo()
    } else {
        serde_json::from_str(&config_json).map_err(|e| err(format!("config: {e}")))?
    };

    let window = web_sys::window().ok_or_else(|| err("no window".into()))?;
    let canvas = window
        .document()
        .and_then(|d| d.get_element_by_id(&canvas_id))
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
        .ok_or_else(|| err(format!("no <canvas id=\"{canvas_id}\">")))?;

    let song_bytes = if song_url.is_empty() {
        None
    } else {
        let bytes = fetch_bytes(&song_url).await;
        if bytes.is_none() {
            log::warn!("could not fetch {song_url}; playing the bundled demo");
        }
        bytes
    };
    let (player, bank) = prepare(song_bytes.as_deref()).map_err(err)?;

    let mut gfx = Canvas2d::new(&canvas, &config).await.map_err(err)?;
    let size = gfx.begin_frame(&canvas);
    let app = App::new(
        Theme::from_theme_config(&config.theme),
        player,
        size,
        config.initial_volume.unwrap_or(0.5),
        (now_ms() % 1_000_000.0) as f32 / 1000.0,
        &gfx,
    );

    let state = Rc::new(RefCell::new(State {
        canvas: canvas.clone(),
        gfx,
        app,
        bank,
        audio: Box::new(NullBackend),
        audio_ready: false,
        input: InputState::default(),
        last_ms: now_ms(),
    }));

    let target: &web_sys::EventTarget = canvas.as_ref();
    listen::<MouseEvent>(target, "mousemove", &state, |s, e| {
        s.input.mouse_pos = Vec2::new(e.offset_x() as f32, e.offset_y() as f32);
    });
    listen::<MouseEvent>(target, "mousedown", &state, |s, e| {
        if e.button() == 0 {
            s.ensure_audio();
            s.input.mouse_pos = Vec2::new(e.offset_x() as f32, e.offset_y() as f32);
            s.input.mouse_down = true;
            s.input.mouse_pressed = true;
        }
    });
    listen::<MouseEvent>(target, "mouseup", &state, |s, e| {
        if e.button() == 0 {
            s.input.mouse_down = false;
            s.input.mouse_released = true;
        }
    });
    let window_target: &web_sys::EventTarget = window.as_ref();
    listen::<KeyboardEvent>(window_target, "keydown", &state, |s, e| {
        if e.repeat() {
            return;
        }
        s.ensure_audio();
        match e.code().as_str() {
            "Space" => {
                e.prevent_default();
                s.input.space_pressed = true;
            }
            "F3" => {
                e.prevent_default();
                s.input.toggle_profiler = true;
            }
            "F4" => {
                e.prevent_default();
                s.input.dump_profile = true;
            }
            _ => {}
        }
    });

    // requestAnimationFrame loop
    let tick: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let next = tick.clone();
    let loop_state = state.clone();
    *next.borrow_mut() = Some(Closure::new(move |ms: f64| {
        loop_state.borrow_mut().frame(ms);
        if let (Some(w), Some(cb)) = (web_sys::window(), tick.borrow().as_ref()) {
            let _ = w.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    if let Some(cb) = next.borrow().as_ref() {
        window
            .request_animation_frame(cb.as_ref().unchecked_ref())
            .map_err(|e| err(format!("requestAnimationFrame: {e:?}")))?;
    }
    Ok(())
}
