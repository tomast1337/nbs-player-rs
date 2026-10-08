//! Renderer- and audio-agnostic core of the NBS player.
//!
//! Nothing here touches a window, a GPU or an audio device. Frontends feed it input and
//! time, then draw the sprites it produces and play the notes it releases through an
//! [`audio::AudioBackend`].

pub mod audio;
pub mod config;
pub mod controls;
pub mod notes;
pub mod piano;
pub mod player;
pub mod profiler;
pub mod song;
pub mod theme;
pub mod types;
pub mod utils;
