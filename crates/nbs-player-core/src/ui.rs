//! Minimal immediate-mode widgets (button, slider) drawn through [`Renderer`].
//! Replaces raygui so every frontend gets identical controls.

use crate::render::{InputState, Renderer};
use crate::theme::Theme;
use crate::types::{Rect, Rgba};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderStyle {
    /// A small handle that moves along the track.
    Handle,
    /// The track fills up to the value.
    Bar,
}

const HANDLE_W: f32 = 16.0;

#[derive(Debug, Default)]
pub struct Ui {
    input: InputState,
    /// Widget currently being pressed/dragged.
    active: Option<u32>,
}

impl Ui {
    pub fn begin_frame(&mut self, input: &InputState) {
        self.input = *input;
        // Keep the active widget through the release frame so its click can register.
        if !input.mouse_down && !input.mouse_released {
            self.active = None;
        }
    }

    fn hovered(&self, rect: Rect) -> bool {
        rect.contains(self.input.mouse_pos)
    }

    /// Invisible button with an accent outline on hover/press. Returns true on click
    /// (press and release both inside the button).
    pub fn button(&mut self, id: u32, rect: Rect, theme: &Theme, r: &mut dyn Renderer) -> bool {
        let hovered = self.hovered(rect);
        if hovered && self.input.mouse_pressed {
            self.active = Some(id);
        }
        let pressed = self.active == Some(id);
        let clicked = hovered && pressed && self.input.mouse_released;

        if hovered || pressed {
            r.draw_rect_outline(rect, 1.0, theme.accent_color);
        }
        clicked
    }

    /// Slider over `min..=max`. Returns true when the user changed `value` this frame.
    pub fn slider(
        &mut self,
        id: u32,
        rect: Rect,
        value: &mut f32,
        min: f32,
        max: f32,
        style: SliderStyle,
        theme: &Theme,
        r: &mut dyn Renderer,
    ) -> bool {
        let hovered = self.hovered(rect);
        if hovered && self.input.mouse_pressed {
            self.active = Some(id);
        }
        let dragging = self.active == Some(id) && self.input.mouse_down;

        let mut changed = false;
        if dragging && max > min {
            let track = (rect.w - if style == SliderStyle::Handle { HANDLE_W } else { 0.0 }).max(1.0);
            let offset = if style == SliderStyle::Handle { HANDLE_W / 2.0 } else { 0.0 };
            let t = ((self.input.mouse_pos.x - rect.x - offset) / track).clamp(0.0, 1.0);
            let new = min + t * (max - min);
            if new != *value {
                *value = new;
                changed = true;
            }
        }

        let track_color: Rgba = if dragging {
            theme.white_key_color.brightness(0.9)
        } else if hovered {
            theme.black_key_color
        } else {
            theme.background_color
        };
        r.draw_rect(rect, track_color);
        if hovered || dragging {
            r.draw_rect_outline(rect, 1.0, theme.accent_color);
        }

        let t = if max > min {
            ((*value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let inner = Rect::new(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, rect.h - 2.0);
        match style {
            SliderStyle::Handle => {
                let x = inner.x + t * (inner.w - HANDLE_W);
                r.draw_rect(Rect::new(x, inner.y, HANDLE_W, inner.h), theme.white_key_color);
            }
            SliderStyle::Bar => {
                r.draw_rect(
                    Rect::new(inner.x, inner.y, inner.w * t, inner.h),
                    theme.white_key_color,
                );
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TextMeasure, Vec2};

    /// Renderer that records nothing; widgets only need it to accept draw calls.
    struct Null;
    impl TextMeasure for Null {
        fn measure_text(&self, _: &str, _: f32) -> Vec2 {
            Vec2::ZERO
        }
    }
    impl Renderer for Null {
        fn draw_background(&mut self, _: f32, _: Vec2, _: &Theme) {}
        fn draw_rect(&mut self, _: Rect, _: Rgba) {}
        fn draw_rect_outline(&mut self, _: Rect, _: f32, _: Rgba) {}
        fn draw_sprite(&mut self, _: crate::render::Sprite, _: Rect, _: Rgba) {}
        fn draw_text(&mut self, _: &str, _: Vec2, _: f32, _: Rgba) {}
    }

    fn input(x: f32, y: f32, down: bool, pressed: bool, released: bool) -> InputState {
        InputState {
            mouse_pos: Vec2::new(x, y),
            mouse_down: down,
            mouse_pressed: pressed,
            mouse_released: released,
            ..Default::default()
        }
    }

    const R: Rect = Rect::new(10.0, 10.0, 100.0, 20.0);

    #[test]
    fn click_needs_press_and_release_inside() {
        let (theme, mut r, mut ui) = (Theme::default(), Null, Ui::default());
        ui.begin_frame(&input(20.0, 15.0, true, true, false));
        assert!(!ui.button(1, R, &theme, &mut r));
        ui.begin_frame(&input(20.0, 15.0, false, false, true));
        assert!(ui.button(1, R, &theme, &mut r));
    }

    #[test]
    fn release_elsewhere_is_not_a_click() {
        let (theme, mut r, mut ui) = (Theme::default(), Null, Ui::default());
        ui.begin_frame(&input(20.0, 15.0, true, true, false));
        ui.button(1, R, &theme, &mut r);
        ui.begin_frame(&input(500.0, 500.0, false, false, true));
        assert!(!ui.button(1, R, &theme, &mut r));
    }

    #[test]
    fn slider_drag_maps_mouse_to_value() {
        let (theme, mut r, mut ui) = (Theme::default(), Null, Ui::default());
        let mut v = 0.0;
        ui.begin_frame(&input(60.0, 15.0, true, true, false));
        assert!(ui.slider(1, R, &mut v, 0.0, 1.0, SliderStyle::Bar, &theme, &mut r));
        assert!((v - 0.5).abs() < 1e-3);
        // dragging outside the rect keeps tracking and clamps
        ui.begin_frame(&input(900.0, 400.0, true, false, false));
        ui.slider(1, R, &mut v, 0.0, 1.0, SliderStyle::Bar, &theme, &mut r);
        assert_eq!(v, 1.0);
        ui.begin_frame(&input(900.0, 400.0, false, false, true));
        assert!(!ui.slider(1, R, &mut v, 0.0, 1.0, SliderStyle::Bar, &theme, &mut r));
    }

    #[test]
    fn slider_without_press_does_not_change() {
        let (theme, mut r, mut ui) = (Theme::default(), Null, Ui::default());
        let mut v = 0.25;
        ui.begin_frame(&input(60.0, 15.0, true, false, false)); // held from outside
        assert!(!ui.slider(1, R, &mut v, 0.0, 1.0, SliderStyle::Bar, &theme, &mut r));
        assert_eq!(v, 0.25);
    }
}
