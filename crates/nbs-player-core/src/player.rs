//! Playback state machine: clock, pause/seek/restart and "which tick fires now".
//! Knows nothing about audio or rendering.

use std::collections::HashMap;

use crate::notes::NoteBlock;
use crate::types::Rgba;

#[derive(Debug, Clone)]
pub struct Player {
    pub note_blocks: Vec<Vec<NoteBlock>>,
    pub instrument_colors: HashMap<u32, Rgba>,
    pub title: String,
    pub song_length: usize,
    pub notes_per_second: f32,
    pub total_duration: f32,

    pub elapsed_time: f32,
    pub current_tick: f32,
    pub is_paused: bool,
    pub is_end: bool,

    /// Last tick whose notes were handed out, so each tick fires once.
    last_fired_tick: Option<usize>,
}

impl Player {
    /// Starts paused at 0:00.
    pub fn new(
        title: String,
        tempo: u16,
        song_length: usize,
        note_blocks: Vec<Vec<NoteBlock>>,
        instrument_colors: HashMap<u32, Rgba>,
    ) -> Self {
        let notes_per_second = tempo as f32 / 100.;
        Self {
            note_blocks,
            instrument_colors,
            title,
            song_length,
            notes_per_second,
            total_duration: song_length as f32 / notes_per_second,
            elapsed_time: 0.0,
            current_tick: 0.0,
            is_paused: true,
            is_end: false,
            last_fired_tick: None,
        }
    }

    /// Advance the clock by `dt` seconds (no-op while paused or finished).
    pub fn update(&mut self, dt: f32) {
        if !self.is_paused && self.elapsed_time < self.total_duration {
            self.elapsed_time += dt;
        }
        self.current_tick = self.elapsed_time * self.notes_per_second;
        self.is_end = self.elapsed_time >= self.total_duration;
    }

    /// Notes of the current tick if it has not fired yet and the song is playing.
    pub fn take_due_notes(&mut self) -> Option<&[NoteBlock]> {
        if self.is_paused || self.elapsed_time >= self.total_duration {
            return None;
        }
        let tick = self.current_tick as usize;
        if self.last_fired_tick == Some(tick) {
            return None;
        }
        self.last_fired_tick = Some(tick);
        self.note_blocks.get(tick).map(Vec::as_slice)
    }

    /// Notes that are under the playhead right now, for key animation.
    pub fn notes_at_playhead(&self) -> Option<&[NoteBlock]> {
        self.note_blocks
            .get(self.current_tick as usize)
            .map(Vec::as_slice)
    }

    pub fn play(&mut self) {
        self.is_paused = false;
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Space bar: when the song has ended it rewinds and ends up paused, otherwise it
    /// toggles pause.
    pub fn space_pressed(&mut self) {
        if self.elapsed_time >= self.total_duration {
            self.rewind();
            self.is_paused = false;
        }
        self.toggle_pause();
    }

    /// Back to 0:00, paused (reset button).
    pub fn reset(&mut self) {
        self.rewind();
        self.is_paused = true;
    }

    /// Jump to `tick` (fractional ticks allowed). The tick at the new position fires again.
    pub fn seek_to_tick(&mut self, tick: f32) {
        let tick = tick.clamp(0.0, self.song_length as f32);
        self.current_tick = tick;
        self.elapsed_time = tick / self.notes_per_second;
        self.last_fired_tick = None;
    }

    fn rewind(&mut self) {
        self.elapsed_time = 0.0;
        self.current_tick = 0.0;
        self.is_end = false;
        self.last_fired_tick = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note() -> NoteBlock {
        NoteBlock {
            instrument: 0,
            key: 60,
            frequency_ratio: 1.0,
            volume: 1.0,
            pan: 0.0,
        }
    }

    // 10 ticks at 10 ticks/s (tempo 1000), a note on every tick.
    fn player() -> Player {
        Player::new(
            "t".into(),
            1000,
            10,
            (0..10).map(|_| vec![note()]).collect(),
            HashMap::new(),
        )
    }

    #[test]
    fn paused_player_does_not_advance_or_fire() {
        let mut p = player();
        p.update(0.5);
        assert_eq!(p.elapsed_time, 0.0);
        assert!(p.take_due_notes().is_none());
    }

    #[test]
    fn each_tick_fires_once() {
        let mut p = player();
        p.play();
        p.update(0.0);
        assert!(p.take_due_notes().is_some());
        assert!(p.take_due_notes().is_none());
        p.update(0.05);
        assert!(p.take_due_notes().is_none()); // still tick 0
        p.update(0.06);
        assert!(p.take_due_notes().is_some()); // tick 1
    }

    #[test]
    fn seek_backwards_refires() {
        let mut p = player();
        p.play();
        p.update(0.55);
        assert!(p.take_due_notes().is_some());
        p.seek_to_tick(2.0);
        p.update(0.0);
        assert_eq!(p.current_tick as usize, 2);
        assert!(p.take_due_notes().is_some());
    }

    #[test]
    fn ends_and_space_rewinds_paused() {
        let mut p = player();
        p.play();
        p.update(2.0);
        assert!(p.is_end);
        assert!(p.take_due_notes().is_none());
        p.space_pressed();
        assert!(!p.is_end);
        assert!(p.is_paused);
        assert_eq!(p.elapsed_time, 0.0);
    }

    #[test]
    fn reset_pauses_at_zero() {
        let mut p = player();
        p.play();
        p.update(0.5);
        p.reset();
        assert!(p.is_paused);
        assert_eq!(p.elapsed_time, 0.0);
    }
}
