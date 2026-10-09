//! Drawing and input abstractions. The core describes a frame through [`Renderer`] and
//! reads the pointer/keyboard through [`InputState`]; frontends implement/fill them.

use crate::types::{Rect, Rgba, TextMeasure, Vec2};

/// Images the core asks the renderer to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sprite {
    Note,
    PianoKey,
    Play,
    Pause,
    Reset,
    Fullscreen,
    Volume0,
    Volume25,
    Volume50,
    Volume75,
    Volume100,
}

impl Sprite {
    /// Every sprite, for frontends that load them all up front.
    pub const ALL: [Sprite; 11] = [
        Sprite::Note,
        Sprite::PianoKey,
        Sprite::Play,
        Sprite::Pause,
        Sprite::Reset,
        Sprite::Fullscreen,
        Sprite::Volume0,
        Sprite::Volume25,
        Sprite::Volume50,
        Sprite::Volume75,
        Sprite::Volume100,
    ];

    /// The bundled PNG data for this sprite.
    pub fn png_bytes(self) -> &'static [u8] {
        match self {
            Sprite::Note => include_bytes!("../../../assets/textures/note_block.png"),
            Sprite::PianoKey => include_bytes!("../../../assets/textures/key_grey.png"),
            Sprite::Play => include_bytes!("../../../assets/textures/play.png"),
            Sprite::Pause => include_bytes!("../../../assets/textures/pause.png"),
            Sprite::Reset => include_bytes!("../../../assets/textures/reset.png"),
            Sprite::Fullscreen => include_bytes!("../../../assets/textures/fullscreen.png"),
            Sprite::Volume0 => include_bytes!("../../../assets/textures/vol000.png"),
            Sprite::Volume25 => include_bytes!("../../../assets/textures/vol025.png"),
            Sprite::Volume50 => include_bytes!("../../../assets/textures/vol050.png"),
            Sprite::Volume75 => include_bytes!("../../../assets/textures/vol075.png"),
            Sprite::Volume100 => include_bytes!("../../../assets/textures/vol100.png"),
        }
    }
}

/// Immediate-mode 2D drawing target for one frame. Coordinates are window pixels with
/// the origin at the top-left. Text measurement comes from [`TextMeasure`].
pub trait Renderer: TextMeasure {
    /// Fill the window with the animated background. `time` is in seconds.
    fn draw_background(&mut self, time: f32, resolution: Vec2, theme: &crate::theme::Theme);
    fn draw_rect(&mut self, rect: Rect, color: Rgba);
    fn draw_rect_outline(&mut self, rect: Rect, thickness: f32, color: Rgba);
    /// Draw a whole sprite stretched into `dst`, multiplied by `tint`.
    fn draw_sprite(&mut self, sprite: Sprite, dst: Rect, tint: Rgba);
    fn draw_text(&mut self, text: &str, pos: Vec2, font_size: f32, color: Rgba);
}

/// Pointer and keyboard state for one frame, filled in by the frontend.
#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    pub mouse_pos: Vec2,
    /// Left button currently held.
    pub mouse_down: bool,
    /// Left button went down this frame.
    pub mouse_pressed: bool,
    /// Left button went up this frame.
    pub mouse_released: bool,
    pub space_pressed: bool,
    pub toggle_profiler: bool,
    pub dump_profile: bool,
}
