use nbs_rs::NbsFile;
use raylib::prelude::*;
use std::collections::HashMap;

use crate::{
    audio::AudioEngine,
    note::NoteBlock,
    piano::{PianoKey, PianoProps},
};

pub struct SongPlayer {
    window_width: f32,
    window_height: f32,
    nbs_file: NbsFile,
    song_name: String,
    song_author: String,
    title: String,
    notes_per_second: f32,
    total_duration: f32,
    rl: RaylibHandle,
    thread: RaylibThread,
    all_keys: Vec<PianoKey>,
    key_map: HashMap<u8, usize>,
    note_blocks: Vec<Vec<NoteBlock>>,
    piano_props: PianoProps,
    note_texture: Texture2D,
    audio_engine: AudioEngine,
    current_tick: f32,
    elapsed_time: f32,
    note_dim: f32,
    key_spacing: f32,
    played_ticks: Vec<bool>,
    instrument_colors: HashMap<u8, &'static str>,
    is_paused: bool,
}

fn native(nbs_file: NbsFile, window_width: Option<f32>, window_height: Option<f32>) {}

fn web(nbs_file: NbsFile, window_width: Option<f32>, window_height: Option<f32>) {}
