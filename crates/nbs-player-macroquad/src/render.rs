//! macroquad implementation of the core's `Renderer`.

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, PipelineParams};
use macroquad::prelude::*;
use nbs_player_core::config::BackgroundType;
use nbs_player_core::render::{InputState, Renderer, Sprite};
use nbs_player_core::theme::Theme;
use nbs_player_core::types::{Rect, Rgba, TextMeasure, Vec2 as CoreVec2};

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const FRAGMENT_HEADER: &str = "#version 100\nprecision mediump float;\n";

const COLOR_UNIFORMS: [&str; 7] = [
    "background_color",
    "accent_color",
    "text_color",
    "white_key_color",
    "black_key_color",
    "white_text_key_color",
    "black_text_key_color",
];

fn color(c: Rgba) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

pub struct MacroquadRenderer {
    font: Font,
    sprites: Vec<Texture2D>,
    background: Material,
    speed: f32,
}

impl MacroquadRenderer {
    pub fn new(font_ttf: &[u8], background: &BackgroundType) -> Result<Self, String> {
        let font = load_ttf_font_from_bytes(font_ttf).map_err(|e| format!("font: {e}"))?;

        let sprites = Sprite::ALL
            .iter()
            .map(|s| {
                let tex = Texture2D::from_file_with_format(s.png_bytes(), None);
                tex.set_filter(FilterMode::Nearest);
                tex
            })
            .collect();

        let mut uniforms = vec![
            UniformDesc::new("iResolution", UniformType::Float2),
            UniformDesc::new("iTime", UniformType::Float1),
            UniformDesc::new("speed", UniformType::Float1),
        ];
        uniforms.extend(
            COLOR_UNIFORMS
                .iter()
                .map(|n| UniformDesc::new(n, UniformType::Float3)),
        );
        let fragment = format!("{FRAGMENT_HEADER}{}", background.fragment_source());
        let background_material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: &fragment,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                uniforms,
                textures: vec![],
            },
        )
        .map_err(|e| format!("background shader: {e}"))?;

        Ok(Self {
            font,
            sprites,
            background: background_material,
            speed: background.speed(),
        })
    }

    fn texture(&self, sprite: Sprite) -> &Texture2D {
        let i = Sprite::ALL.iter().position(|s| *s == sprite).unwrap();
        &self.sprites[i]
    }
}

/// macroquad rasterizes fonts at integer sizes; draw at the nearest one and scale the rest.
fn split_size(font_size: f32) -> (u16, f32) {
    let px = font_size.round().max(1.0);
    (px as u16, font_size / px)
}

impl TextMeasure for MacroquadRenderer {
    fn measure_text(&self, text: &str, font_size: f32) -> CoreVec2 {
        let (px, scale) = split_size(font_size);
        let d = measure_text(text, Some(&self.font), px, scale);
        CoreVec2::new(d.width, font_size)
    }
}

impl Renderer for MacroquadRenderer {
    fn draw_background(&mut self, time: f32, resolution: CoreVec2, theme: &Theme) {
        let m = &self.background;
        m.set_uniform("iResolution", [resolution.x, resolution.y]);
        m.set_uniform("iTime", time);
        m.set_uniform("speed", self.speed);
        let colors = [
            theme.background_color,
            theme.accent_color,
            theme.text_color,
            theme.white_key_color,
            theme.black_key_color,
            theme.white_text_key_color,
            theme.black_text_key_color,
        ];
        for (name, c) in COLOR_UNIFORMS.iter().zip(colors) {
            m.set_uniform(name, c.to_f32_rgb());
        }
        gl_use_material(m);
        draw_rectangle(0.0, 0.0, resolution.x, resolution.y, WHITE);
        gl_use_default_material();
    }

    fn draw_rect(&mut self, r: Rect, c: Rgba) {
        draw_rectangle(r.x, r.y, r.w, r.h, color(c));
    }

    fn draw_rect_outline(&mut self, r: Rect, thickness: f32, c: Rgba) {
        // macroquad centers the stroke on the edge; raylib draws it inward.
        let h = thickness / 2.0;
        draw_rectangle_lines(
            r.x + h,
            r.y + h,
            r.w - thickness,
            r.h - thickness,
            thickness,
            color(c),
        );
    }

    fn draw_sprite(&mut self, sprite: Sprite, dst: Rect, tint: Rgba) {
        draw_texture_ex(
            self.texture(sprite),
            dst.x,
            dst.y,
            color(tint),
            DrawTextureParams {
                dest_size: Some(vec2(dst.w, dst.h)),
                ..Default::default()
            },
        );
    }

    fn draw_text(&mut self, text: &str, pos: CoreVec2, font_size: f32, c: Rgba) {
        let (px, scale) = split_size(font_size);
        // macroquad positions text by its baseline; the core passes the top-left corner.
        let ascent = measure_text(text, Some(&self.font), px, scale).offset_y;
        draw_text_ex(
            text,
            pos.x,
            pos.y + ascent,
            TextParams {
                font: Some(&self.font),
                font_size: px,
                font_scale: scale,
                color: color(c),
                ..Default::default()
            },
        );
    }
}

/// Snapshot the mouse and the keys the core cares about.
pub fn read_input() -> InputState {
    let (x, y) = mouse_position();
    let left = MouseButton::Left;
    InputState {
        mouse_pos: CoreVec2::new(x, y),
        mouse_down: is_mouse_button_down(left),
        mouse_pressed: is_mouse_button_pressed(left),
        mouse_released: is_mouse_button_released(left),
        space_pressed: is_key_pressed(KeyCode::Space),
        toggle_profiler: is_key_pressed(KeyCode::F3),
        dump_profile: is_key_pressed(KeyCode::F4),
    }
}
