//! Control-panel state (auto-hide on mouse inactivity) and the pure layout of its widgets.

use crate::types::{Rect, Vec2};
use crate::utils;

#[derive(Debug, Clone)]
pub struct ControlsState {
    pub controls_close_time: f32,      // Seconds of mouse inactivity before the panel hides
    pub sec_since_last_mouse_move: f32, // Timer for mouse inactivity
    pub last_mouse_pos: Vec2,          // Last recorded mouse position
    pub controls_panel_y: f32,         // Current top of the panel (animated)
    pub button_size: Vec2,             // Size of buttons
    pub control_panel_height: f32,     // Height of the control panel
    pub timeline_height: f32,          // Height of the timeline slider
    pub toggle_fullscreen: bool,       // Fullscreen request flag
}

/// Screen rectangles of every control for the current frame.
#[derive(Debug, Clone, Copy)]
pub struct ControlsLayout {
    pub panel: Rect,
    pub play_pause: Rect,
    pub reset: Rect,
    pub timeline: Rect,
    pub volume: Rect,
    pub fullscreen: Rect,
    /// Big play button shown in the middle of the screen while paused.
    pub center_play: Rect,
}

impl ControlsState {
    pub fn new(window_height: f32) -> Self {
        const BTN_S: f32 = 40.;
        let button_size = Vec2::new(BTN_S, BTN_S);
        let control_panel_height = button_size.y * 2.0;
        ControlsState {
            controls_close_time: 0.5,
            sec_since_last_mouse_move: 0.0,
            last_mouse_pos: Vec2::ZERO,
            button_size,
            control_panel_height,
            timeline_height: button_size.y,
            toggle_fullscreen: false,
            controls_panel_y: window_height - control_panel_height,
        }
    }

    pub fn update_mouse_state(&mut self, current_mouse_pos: Vec2, delta_time: f32) {
        if current_mouse_pos != self.last_mouse_pos {
            self.sec_since_last_mouse_move = 0.0;
            self.last_mouse_pos = current_mouse_pos;
        } else {
            self.sec_since_last_mouse_move += delta_time;
        }
    }

    pub fn update_controls_panel_position(&mut self, window_height: f32) {
        let target = if self.sec_since_last_mouse_move < self.controls_close_time {
            window_height - self.control_panel_height // slide up (show)
        } else {
            window_height // slide down (hide)
        };
        self.controls_panel_y = utils::lerp(self.controls_panel_y, target, 0.2);
    }

    pub fn layout(&self, window_width: f32, window_height: f32) -> ControlsLayout {
        let y = self.controls_panel_y;
        let h = self.control_panel_height;
        let b = self.button_size;
        let mid = y + h / 2.0;

        let panel = Rect::new(0.0, y, window_width, h);
        let play_pause = Rect::new(b.x / 2., mid - b.y / 2.0, b.x, b.y);
        let reset = Rect::new(b.x + b.x / 2. + 10.0, mid - b.y / 2.0, b.x, b.y);
        let timeline = Rect::new(
            reset.x + reset.w + 10.0,
            mid - self.timeline_height / 2.0,
            window_width - reset.w - b.x * 5.5 - 30.0,
            self.timeline_height,
        );
        let volume = Rect::new(timeline.x + timeline.w + 10.0, mid - b.y / 2.0, b.x * 2.0, b.y);
        let fullscreen = Rect::new(volume.x + volume.w + 10.0, volume.y, b.x, b.y);
        let center_play = Rect::new(window_width / 2. - 50., window_height / 2. - 50., 100., 100.);

        ControlsLayout {
            panel,
            play_pause,
            reset,
            timeline,
            volume,
            fullscreen,
            center_play,
        }
    }

    /// Keep the panel open while the pointer rests on it.
    pub fn hold_open_if_hovered(&mut self, panel: Rect) {
        if panel.contains(self.last_mouse_pos) {
            self.sec_since_last_mouse_move = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_hides_after_inactivity_and_returns_on_motion() {
        let mut c = ControlsState::new(720.0);
        for _ in 0..120 {
            c.update_mouse_state(Vec2::new(5.0, 5.0), 1.0 / 60.0);
            c.update_controls_panel_position(720.0);
        }
        assert!(c.controls_panel_y > 719.0);
        c.update_mouse_state(Vec2::new(9.0, 9.0), 1.0 / 60.0);
        for _ in 0..60 {
            c.update_controls_panel_position(720.0);
        }
        assert!(c.controls_panel_y < 641.0);
    }

    #[test]
    fn layout_fits_window() {
        let c = ControlsState::new(720.0);
        let l = c.layout(1280.0, 720.0);
        assert!(l.fullscreen.x + l.fullscreen.w <= 1280.0);
        assert!(l.timeline.w > 0.0);
    }
}
