//! Browser entry point, exported through wasm-bindgen.

use nbs_player_core::config::AppConfig;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::web::EventLoopExtWebSys;

use crate::{handler, prepare, Pending};

async fn fetch_bytes(url: &str) -> Option<Vec<u8>> {
    let window = web_sys::window()?;
    let response = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url)).await.ok()?;
    let response: web_sys::Response = response.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let buffer = wasm_bindgen_futures::JsFuture::from(response.array_buffer().ok()?).await.ok()?;
    Some(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// Start the player on the `<canvas id="canvas_id">`.
/// `config_json` is an `AppConfig` (empty string = demo theme); `song_url` is fetched as a
/// `.nbs` or zip (empty or failing = the bundled demo song).
#[wasm_bindgen]
pub async fn start(canvas_id: String, config_json: String, song_url: String) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);

    let config: AppConfig = if config_json.trim().is_empty() {
        AppConfig::demo()
    } else {
        serde_json::from_str(&config_json).map_err(|e| JsValue::from_str(&format!("config: {e}")))?
    };

    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(&canvas_id))
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| JsValue::from_str(&format!("no <canvas id=\"{canvas_id}\">")))?;

    let song = if song_url.is_empty() {
        None
    } else {
        let bytes = fetch_bytes(&song_url).await;
        if bytes.is_none() {
            log::warn!("could not fetch {song_url}; playing the bundled demo");
        }
        bytes
    };
    let (player, bank) = prepare(&config, song.as_deref()).map_err(|e| JsValue::from_str(&e))?;

    let event_loop = EventLoop::new().map_err(|e| JsValue::from_str(&e.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.spawn_app(handler(Pending {
        config,
        player,
        bank,
        canvas,
    }));
    Ok(())
}
