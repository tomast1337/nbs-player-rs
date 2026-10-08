//! raylib glue: converts core types and draws the sprites the core produces.

use nbs_player_core::notes::{self, NoteScene};
use nbs_player_core::piano;
use nbs_player_core::types::{Rect, Rgba, TextMeasure, Vec2};
use raylib::prelude::*;

use crate::app_state::AppState;

pub fn color(c: Rgba) -> Color {
    Color::new(c.r, c.g, c.b, c.a)
}

pub fn rect(r: Rect) -> Rectangle {
    Rectangle::new(r.x, r.y, r.w, r.h)
}

pub fn vec2(v: Vec2) -> Vector2 {
    Vector2::new(v.x, v.y)
}

pub fn from_vec2(v: Vector2) -> Vec2 {
    Vec2::new(v.x, v.y)
}

/// Lets the core measure text with a raylib font.
pub struct RlText<'a>(pub &'a Font);

impl TextMeasure for RlText<'_> {
    fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
        from_vec2(self.0.measure_text(text, font_size, 0.0))
    }
}

/// Full-texture source rectangle.
pub fn tex_src(t: &Texture2D) -> Rectangle {
    Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32)
}

/// Draws the falling notes. Returns how many were on screen.
pub fn draw_notes(d: &mut RaylibDrawHandle<'_>, app: &AppState) -> usize {
    let scene = NoteScene {
        window_width: app.window_width,
        window_height: app.window_height,
        keys: &app.piano_state.all_keys,
        key_map: &app.piano_state.key_map,
        piano_props: &app.piano_state.piano_props,
        note_blocks: &app.player.note_blocks,
        current_tick: app.player.current_tick,
        note_dim: app.note_dim,
        key_spacing: app.key_spacing,
        instrument_colors: &app.player.instrument_colors,
        font_size_2: app.font_size_2,
        font_size_3: app.font_size_3,
    };
    let note_texture = &app.textures.note_texture;
    let src = tex_src(note_texture);
    let font = &app.font;

    notes::visible_notes(&scene, &RlText(font), |n| {
        d.draw_texture_pro(
            note_texture,
            src,
            rect(n.rect),
            Vector2::zero(),
            0.0,
            color(n.color),
        );
        d.draw_text_pro(
            font,
            n.label,
            vec2(n.label_pos),
            Vector2::new(0.5, 0.5),
            0.0,
            n.font_size,
            0.0,
            Color::WHITE,
        );
    })
}

pub fn draw_piano_keys(d: &mut RaylibDrawHandle<'_>, app: &AppState) {
    let key_texture = &app.textures.piano_key_texture;
    let src = tex_src(key_texture);
    let font = &app.font;

    d.draw_rectangle_rec(
        rect(app.piano_state.bounds(app.window_width, app.window_height)),
        Color::BLACK,
    );
    piano::for_each_key_sprite(
        &app.piano_state,
        app.window_width,
        app.window_height,
        &app.theme,
        &RlText(font),
        |k| {
            d.draw_texture_pro(
                key_texture,
                src,
                rect(k.rect),
                Vector2::zero(),
                0.0,
                color(k.color),
            );
            d.draw_text_pro(
                font,
                k.label,
                vec2(k.label_pos),
                Vector2::zero(),
                0.0,
                k.font_size,
                0.,
                color(k.text_color),
            );
        },
    );
}
