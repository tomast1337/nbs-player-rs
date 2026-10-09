//! On-screen view of the profiler (avg / peak ms, call count, budget bar), drawn through
//! the [`Renderer`] so it looks the same in every frontend.

use crate::profiler;
use crate::render::Renderer;
use crate::types::{Rect, Rgba, Vec2};

const ROW_H: f32 = 15.0;
const PANEL_W: f32 = 400.0;
const FONT: f32 = 12.0;
const BAR_W: f32 = 70.0;
/// A full bar is one 60 fps frame budget.
const BUDGET_MS: f32 = 16.67;

pub fn draw(r: &mut dyn Renderer, window_width: f32, frame_time: f32) {
    let rows = profiler::rows();
    if rows.is_empty() {
        return;
    }

    let x = window_width - PANEL_W - 10.0;
    let y = 30.0;
    let height = (rows.len() as f32 + 2.0) * ROW_H + 8.0;
    r.draw_rect(
        Rect::new(x - 4.0, y - 4.0, PANEL_W + 8.0, height),
        Rgba::new(0, 0, 0, 170),
    );
    r.draw_text(
        &format!(
            "frame {:.2} ms ({:.0} fps)  F3 off  F4 dump",
            frame_time * 1000.0,
            1.0 / frame_time.max(1e-6)
        ),
        Vec2::new(x, y),
        FONT,
        Rgba::WHITE,
    );

    for (i, row) in rows.iter().enumerate() {
        let ry = y + (i as f32 + 1.0) * ROW_H + 4.0;
        r.draw_text(
            &format!("{}{}", "  ".repeat(row.depth), row.name),
            Vec2::new(x, ry),
            FONT,
            Rgba::new(200, 200, 200, 255),
        );
        r.draw_text(
            &format!("{:6.3} / {:6.3}ms x{}", row.avg_ms, row.peak_ms, row.calls),
            Vec2::new(x + 150.0, ry),
            FONT,
            Rgba::WHITE,
        );
        let frac = (row.avg_ms / BUDGET_MS).clamp(0.0, 1.0);
        let color = if frac > 0.5 {
            Rgba::new(230, 41, 55, 255)
        } else if frac > 0.2 {
            Rgba::new(255, 161, 0, 255)
        } else {
            Rgba::new(0, 228, 48, 255)
        };
        r.draw_rect(
            Rect::new(x + PANEL_W - BAR_W, ry + 2.0, (BAR_W * frac).max(1.0), 8.0),
            color,
        );
    }
}
