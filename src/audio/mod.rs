//! Audio backends for the app. The core crate defines `AudioBackend`; this module picks
//! and constructs an implementation.

mod raylib_backend;

use nbs_player_core::audio::{AudioBackend, InstrumentBank};
use nbs_player_core::config::AudioBackendKind;
use raylib::prelude::RaylibAudio;

pub use raylib_backend::RaylibBackend;

/// Build the requested backend and load every instrument into it.
/// Falls back to raylib (when a device is available) if the requested one fails.
pub fn create_backend<'a>(
    kind: AudioBackendKind,
    raylib_audio: Option<&'a RaylibAudio>,
    bank: &InstrumentBank,
) -> Box<dyn AudioBackend + 'a> {
    let mut backend: Box<dyn AudioBackend + 'a> = match kind {
        AudioBackendKind::Cpal => match create_cpal() {
            Some(b) => b,
            None => {
                log::warn!("cpal audio unavailable, falling back to raylib audio");
                Box::new(RaylibBackend::new(
                    raylib_audio.expect("raylib audio device is required for the fallback"),
                ))
            }
        },
        AudioBackendKind::Raylib => Box::new(RaylibBackend::new(
            raylib_audio.expect("raylib audio device is required"),
        )),
    };
    bank.install(backend.as_mut());
    backend
}

#[cfg(not(target_os = "emscripten"))]
fn create_cpal<'a>() -> Option<Box<dyn AudioBackend + 'a>> {
    match nbs_player_cpal::CpalBackend::new() {
        Ok(b) => Some(Box::new(b)),
        Err(e) => {
            log::error!("cpal init failed: {e}");
            None
        }
    }
}

#[cfg(target_os = "emscripten")]
fn create_cpal<'a>() -> Option<Box<dyn AudioBackend + 'a>> {
    log::warn!("cpal has no emscripten host");
    None
}

/// Whether the raylib audio device must be opened for this configuration.
/// (The cpal path falls back to raylib, so it is opened on emscripten too.)
pub fn needs_raylib_audio(kind: AudioBackendKind) -> bool {
    kind == AudioBackendKind::Raylib || cfg!(target_os = "emscripten")
}
