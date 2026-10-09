//! `Renderer` on a 2D canvas context.

use std::cell::RefCell;
use std::collections::HashMap;

use nbs_player_core::config::{AppConfig, BackgroundType};
use nbs_player_core::render::{Renderer, Sprite};
use nbs_player_core::theme::Theme;
use nbs_player_core::types::{Rect, Rgba, TextMeasure, Vec2};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageBitmap};

const FONT_FAMILY: &str = "nbs-player-font";
/// The animated background is computed at this size and stretched to the window.
const BG_W: u32 = 160;
const BG_H: u32 = 90;

pub struct Canvas2d {
    ctx: CanvasRenderingContext2d,
    sprites: HashMap<Sprite, ImageBitmap>,
    /// Sprites multiplied by a color, made once per (sprite, rgb).
    tinted: RefCell<HashMap<(Sprite, [u8; 3]), HtmlCanvasElement>>,
    background: BackgroundType,
    bg_canvas: HtmlCanvasElement,
    bg_pixels: Vec<u8>,
    dpr: f64,
}

fn css(c: Rgba) -> String {
    format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a as f32 / 255.0)
}

fn js_err(e: JsValue) -> String {
    format!("{e:?}")
}

fn make_canvas(w: u32, h: u32) -> Result<HtmlCanvasElement, String> {
    let canvas: HtmlCanvasElement = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?
        .create_element("canvas")
        .map_err(js_err)?
        .dyn_into()
        .map_err(|_| "not a canvas")?;
    canvas.set_width(w);
    canvas.set_height(h);
    Ok(canvas)
}

fn context(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, String> {
    canvas
        .get_context("2d")
        .map_err(js_err)?
        .ok_or("2d context unavailable")?
        .dyn_into()
        .map_err(|_| "not a 2d context".into())
}

async fn decode_png(bytes: &'static [u8]) -> Result<ImageBitmap, String> {
    let window = web_sys::window().ok_or("no window")?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("image/png");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(js_err)?;
    let promise = window.create_image_bitmap_with_blob(&blob).map_err(js_err)?;
    JsFuture::from(promise)
        .await
        .map_err(js_err)?
        .dyn_into()
        .map_err(|_| "not an ImageBitmap".into())
}

async fn load_font(config: &AppConfig) -> Result<(), String> {
    let mut data = config.font_id.ttf_bytes().to_vec();
    let face = web_sys::FontFace::new_with_u8_array(FONT_FAMILY, &mut data).map_err(js_err)?;
    JsFuture::from(face.load().map_err(js_err)?).await.map_err(js_err)?;
    let document = web_sys::window().and_then(|w| w.document()).ok_or("no document")?;
    document.fonts().add(&face).map_err(js_err)?;
    Ok(())
}

impl Canvas2d {
    pub async fn new(canvas: &HtmlCanvasElement, config: &AppConfig) -> Result<Self, String> {
        let ctx = context(canvas)?;
        load_font(config).await?;
        let mut sprites = HashMap::new();
        for sprite in Sprite::ALL {
            sprites.insert(sprite, decode_png(sprite.png_bytes()).await?);
        }
        Ok(Self {
            ctx,
            sprites,
            tinted: RefCell::new(HashMap::new()),
            background: config.background.clone(),
            bg_canvas: make_canvas(BG_W, BG_H)?,
            bg_pixels: vec![255; (BG_W * BG_H * 4) as usize],
            dpr: 1.0,
        })
    }

    /// Match the canvas backing store to its CSS size and pixel ratio; returns the size in
    /// CSS pixels, which is what the core lays out in.
    pub fn begin_frame(&mut self, canvas: &HtmlCanvasElement) -> Vec2 {
        self.dpr = web_sys::window().map_or(1.0, |w| w.device_pixel_ratio());
        let (w, h) = (canvas.client_width().max(1) as f64, canvas.client_height().max(1) as f64);
        let (pw, ph) = ((w * self.dpr).round() as u32, (h * self.dpr).round() as u32);
        if canvas.width() != pw || canvas.height() != ph {
            canvas.set_width(pw);
            canvas.set_height(ph);
        }
        let _ = self.ctx.set_transform(self.dpr, 0.0, 0.0, self.dpr, 0.0, 0.0);
        // pixel art stays crisp
        self.ctx.set_image_smoothing_enabled(false);
        Vec2::new(w as f32, h as f32)
    }

    fn tinted_sprite(&self, sprite: Sprite, tint: Rgba) -> Option<HtmlCanvasElement> {
        let key = (sprite, [tint.r, tint.g, tint.b]);
        if let Some(c) = self.tinted.borrow().get(&key) {
            return Some(c.clone());
        }
        let bitmap = self.sprites.get(&sprite)?;
        let canvas = make_canvas(bitmap.width(), bitmap.height()).ok()?;
        let ctx = context(&canvas).ok()?;
        let _ = ctx.draw_image_with_image_bitmap(bitmap, 0.0, 0.0);
        // multiply the color in, then restore the sprite's own alpha
        let _ = ctx.set_global_composite_operation("multiply");
        ctx.set_fill_style_str(&css(Rgba::new(tint.r, tint.g, tint.b, 255)));
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        let _ = ctx.set_global_composite_operation("destination-in");
        let _ = ctx.draw_image_with_image_bitmap(bitmap, 0.0, 0.0);
        self.tinted.borrow_mut().insert(key, canvas.clone());
        Some(canvas)
    }
}

fn mix(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

impl Renderer for Canvas2d {
    fn draw_background(&mut self, time: f32, resolution: Vec2, theme: &Theme) {
        let bg = theme.background_color;
        if self.background == BackgroundType::Plain {
            self.ctx.set_fill_style_str(&css(bg));
            self.ctx.fill_rect(0.0, 0.0, resolution.x as f64, resolution.y as f64);
            return;
        }
        // A cheap plasma between the theme's background and accent colors.
        let accent = theme.accent_color;
        let t = time * (0.3 + self.background.speed() * 0.2);
        for y in 0..BG_H {
            for x in 0..BG_W {
                let (fx, fy) = (x as f32, y as f32);
                let v = (fx * 0.09 + t).sin()
                    + (fy * 0.13 - t * 0.7).sin()
                    + ((fx + fy) * 0.06 + t * 1.3).sin()
                    + ((fx * fx + fy * fy).sqrt() * 0.08 - t).sin();
                let k = (v * 0.125 + 0.5).clamp(0.0, 1.0) * 0.55;
                let i = ((y * BG_W + x) * 4) as usize;
                self.bg_pixels[i] = mix(bg.r, accent.r, k);
                self.bg_pixels[i + 1] = mix(bg.g, accent.g, k);
                self.bg_pixels[i + 2] = mix(bg.b, accent.b, k);
            }
        }
        if let Ok(image) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.bg_pixels),
            BG_W,
            BG_H,
        ) {
            if let Ok(ctx) = context(&self.bg_canvas) {
                let _ = ctx.put_image_data(&image, 0.0, 0.0);
            }
        }
        let _ = self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
            &self.bg_canvas,
            0.0,
            0.0,
            resolution.x as f64,
            resolution.y as f64,
        );
    }

    fn draw_rect(&mut self, r: Rect, color: Rgba) {
        self.ctx.set_fill_style_str(&css(color));
        self.ctx.fill_rect(r.x as f64, r.y as f64, r.w as f64, r.h as f64);
    }

    fn draw_rect_outline(&mut self, r: Rect, thickness: f32, color: Rgba) {
        // inside the rect, like the other frontends
        let t = thickness as f64;
        self.ctx.set_stroke_style_str(&css(color));
        self.ctx.set_line_width(t);
        self.ctx.stroke_rect(
            r.x as f64 + t / 2.0,
            r.y as f64 + t / 2.0,
            r.w as f64 - t,
            r.h as f64 - t,
        );
    }

    fn draw_sprite(&mut self, sprite: Sprite, dst: Rect, tint: Rgba) {
        let (x, y, w, h) = (dst.x as f64, dst.y as f64, dst.w as f64, dst.h as f64);
        self.ctx.set_global_alpha(tint.a as f64 / 255.0);
        if (tint.r, tint.g, tint.b) == (255, 255, 255) {
            if let Some(bitmap) = self.sprites.get(&sprite) {
                let _ = self.ctx.draw_image_with_image_bitmap_and_dw_and_dh(bitmap, x, y, w, h);
            }
        } else if let Some(canvas) = self.tinted_sprite(sprite, tint) {
            let _ = self.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(&canvas, x, y, w, h);
        }
        self.ctx.set_global_alpha(1.0);
    }

    fn draw_text(&mut self, text: &str, pos: Vec2, font_size: f32, color: Rgba) {
        self.ctx.set_font(&format!("{font_size}px \"{FONT_FAMILY}\""));
        self.ctx.set_text_baseline("top");
        self.ctx.set_fill_style_str(&css(color));
        let _ = self.ctx.fill_text(text, pos.x as f64, pos.y as f64);
    }
}

impl TextMeasure for Canvas2d {
    fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
        self.ctx.set_font(&format!("{font_size}px \"{FONT_FAMILY}\""));
        let width = self.ctx.measure_text(text).map_or(0.0, |m| m.width());
        Vec2::new(width as f32, font_size)
    }
}
