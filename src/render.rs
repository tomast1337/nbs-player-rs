//! raylib frontend: implements the core's `Renderer` and gathers `InputState`.

use nbs_player_core::config::BackgroundType;
use nbs_player_core::render::{InputState, Renderer, Sprite};
use nbs_player_core::theme::Theme;
use nbs_player_core::types::{Rect, Rgba, TextMeasure, Vec2};
use raylib::prelude::*;

use crate::background::Background;
use crate::textures::Textures;

fn color(c: Rgba) -> Color {
    Color::new(c.r, c.g, c.b, c.a)
}

fn rect(r: Rect) -> Rectangle {
    Rectangle::new(r.x, r.y, r.w, r.h)
}

fn vec2(v: Vec2) -> Vector2 {
    Vector2::new(v.x, v.y)
}

/// GPU resources the renderer draws with (font, textures, background shader).
pub struct Resources {
    pub font: Font,
    pub textures: Textures,
    pub background: Background,
}

impl Resources {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        font: Font,
        background: BackgroundType,
    ) -> Self {
        Self {
            textures: crate::textures::load_textures(rl, thread),
            background: Background::new(rl, thread, background),
            font,
        }
    }

    fn texture(&self, sprite: Sprite) -> &Texture2D {
        let t = &self.textures;
        match sprite {
            Sprite::Note => &t.note_texture,
            Sprite::PianoKey => &t.piano_key_texture,
            Sprite::Play => &t.play_button,
            Sprite::Pause => &t.pause_button,
            Sprite::Reset => &t.reset_button,
            Sprite::Fullscreen => &t.fullscreen_button,
            Sprite::Volume0 => &t.vol_000,
            Sprite::Volume25 => &t.vol_025,
            Sprite::Volume50 => &t.vol_050,
            Sprite::Volume75 => &t.vol_075,
            Sprite::Volume100 => &t.vol_100,
        }
    }
}

impl TextMeasure for Resources {
    fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
        let v = self.font.measure_text(text, font_size, 0.0);
        Vec2::new(v.x, v.y)
    }
}

/// One frame's drawing target: a raylib draw handle plus the shared resources.
pub struct RaylibRenderer<'a, 'd> {
    pub d: &'a mut RaylibDrawHandle<'d>,
    pub res: &'a mut Resources,
}

impl TextMeasure for RaylibRenderer<'_, '_> {
    fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
        self.res.measure_text(text, font_size)
    }
}

impl Renderer for RaylibRenderer<'_, '_> {
    fn draw_background(&mut self, time: f32, resolution: Vec2, theme: &Theme) {
        self.res.background.shader_time = time;
        self.res
            .background
            .draw(self.d, theme, [resolution.x, resolution.y]);
    }

    fn draw_rect(&mut self, r: Rect, c: Rgba) {
        self.d.draw_rectangle_rec(rect(r), color(c));
    }

    fn draw_rect_outline(&mut self, r: Rect, thickness: f32, c: Rgba) {
        self.d.draw_rectangle_lines_ex(rect(r), thickness, color(c));
    }

    fn draw_sprite(&mut self, sprite: Sprite, dst: Rect, tint: Rgba) {
        let tex = self.res.texture(sprite);
        let src = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
        self.d
            .draw_texture_pro(tex, src, rect(dst), Vector2::zero(), 0.0, color(tint));
    }

    fn draw_text(&mut self, text: &str, pos: Vec2, font_size: f32, c: Rgba) {
        self.d.draw_text_pro(
            &self.res.font,
            text,
            vec2(pos),
            Vector2::zero(),
            0.0,
            font_size,
            0.0,
            color(c),
        );
    }
}

/// Snapshot the mouse and the keys the core cares about.
pub fn read_input(rl: &RaylibHandle) -> InputState {
    let m = rl.get_mouse_position();
    let left = MouseButton::MOUSE_BUTTON_LEFT;
    InputState {
        mouse_pos: Vec2::new(m.x, m.y),
        mouse_down: rl.is_mouse_button_down(left),
        mouse_pressed: rl.is_mouse_button_pressed(left),
        mouse_released: rl.is_mouse_button_released(left),
        space_pressed: rl.is_key_pressed(KeyboardKey::KEY_SPACE),
        toggle_profiler: rl.is_key_pressed(KeyboardKey::KEY_F3),
        dump_profile: rl.is_key_pressed(KeyboardKey::KEY_F4),
    }
}
