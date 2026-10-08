//! Backend-neutral geometry and color types.

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const WHITE: Rgba = Rgba::new(255, 255, 255, 255);
    pub const BLACK: Rgba = Rgba::new(0, 0, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parses `rrggbb` / `#rrggbb`. Invalid input yields `None`.
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        if s.len() < 6 || !s.is_char_boundary(6) {
            return None;
        }
        let c = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
        Some(Self::new(c(0)?, c(2)?, c(4)?, 255))
    }

    /// Same as raylib's `Fade`: `a` is clamped to 0..=1 and replaces the alpha channel.
    pub fn alpha(self, a: f32) -> Self {
        Self {
            a: (255.0 * a.clamp(0.0, 1.0)) as u8,
            ..self
        }
    }

    /// Same as raylib's `ColorBrightness`: `factor` in -1..=1.
    pub fn brightness(self, factor: f32) -> Self {
        let f = factor.clamp(-1.0, 1.0);
        let ch = |v: u8| -> u8 {
            let v = v as f32;
            let out = if f < 0.0 { (1.0 + f) * v } else { (255.0 - v) * f + v };
            out.clamp(0.0, 255.0) as u8
        };
        Self::new(ch(self.r), ch(self.g), ch(self.b), self.a)
    }

    /// Same as raylib's `ColorFromHSV` (hue in degrees).
    pub fn from_hsv(hue: f32, saturation: f32, value: f32) -> Self {
        let channel = |offset: f32| -> u8 {
            let mut k = (offset + hue / 60.0) % 6.0;
            let t = 4.0 - k;
            k = if t < k { t } else { k };
            k = if k < 1.0 { k } else { 1.0 };
            k = if k > 0.0 { k } else { 0.0 };
            ((value - value * saturation * k) * 255.0) as u8
        };
        Self::new(channel(5.0), channel(3.0), channel(1.0), 255)
    }

    pub fn to_f32_rgb(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
}

/// Text measurement supplied by the renderer, so layout stays backend-free.
pub trait TextMeasure {
    fn measure_text(&self, text: &str, font_size: f32) -> Vec2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing() {
        assert_eq!(Rgba::from_hex("#1964ac"), Some(Rgba::new(0x19, 0x64, 0xac, 255)));
        assert_eq!(Rgba::from_hex("zz"), None);
        assert_eq!(Rgba::from_hex("ggggggg"), None);
    }

    #[test]
    fn brightness_matches_raylib() {
        assert_eq!(Rgba::BLACK.brightness(0.2), Rgba::new(51, 51, 51, 255));
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(Rgba::from_hsv(0.0, 1.0, 1.0), Rgba::new(255, 0, 0, 255));
        assert_eq!(Rgba::from_hsv(120.0, 1.0, 1.0), Rgba::new(0, 255, 0, 255));
    }

    #[test]
    fn alpha_clamps_like_fade() {
        assert_eq!(Rgba::WHITE.alpha(300.0).a, 255);
        assert_eq!(Rgba::WHITE.alpha(0.0).a, 0);
    }
}
