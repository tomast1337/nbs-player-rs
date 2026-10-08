//! On-screen view of the core profiler: F3 toggles, F4 dumps `profile.folded`.

use nbs_player_core::profiler;
use raylib::prelude::*;

pub fn handle_input(rl: &RaylibHandle) {
    if rl.is_key_pressed(KeyboardKey::KEY_F3) {
        profiler::toggle();
        log::info!(
            "profiler {}",
            if profiler::is_enabled() { "on" } else { "off" }
        );
    }
    if rl.is_key_pressed(KeyboardKey::KEY_F4) {
        profiler::dump_folded();
    }
}

/// Draw the call tree (avg / peak ms, calls, bar) at the top-right.
pub fn draw(d: &mut RaylibDrawHandle<'_>, window_width: f32, frame_time: f32) {
    const ROW_H: i32 = 14;
    const PANEL_W: i32 = 360;
    const BAR_W: f32 = 70.0;
    // A full bar is one 60 fps frame budget.
    const BUDGET_MS: f32 = 16.67;

    let rows = profiler::rows();
    if rows.is_empty() {
        return;
    }

    let x = window_width as i32 - PANEL_W - 10;
    let y = 30;
    let h = (rows.len() as i32 + 2) * ROW_H + 8;
    d.draw_rectangle(x - 4, y - 4, PANEL_W + 8, h, Color::new(0, 0, 0, 170));
    d.draw_text(
        &format!(
            "frame {:.2} ms  ({:.0} fps)   F3 off  F4 dump",
            frame_time * 1000.0,
            1.0 / frame_time.max(1e-6)
        ),
        x,
        y,
        10,
        Color::WHITE,
    );

    for (row, r) in rows.iter().enumerate() {
        let ry = y + (row as i32 + 1) * ROW_H + 4;
        d.draw_text(
            &format!("{}{}", "  ".repeat(r.depth), r.name),
            x,
            ry,
            10,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            &format!("{:6.3} / {:6.3}ms x{}", r.avg_ms, r.peak_ms, r.calls),
            x + 130,
            ry,
            10,
            Color::WHITE,
        );
        let frac = (r.avg_ms / BUDGET_MS).clamp(0.0, 1.0);
        let bar = if frac > 0.5 {
            Color::RED
        } else if frac > 0.2 {
            Color::ORANGE
        } else {
            Color::LIME
        };
        d.draw_rectangle(
            x + PANEL_W - BAR_W as i32,
            ry + 1,
            (BAR_W * frac).max(1.0) as i32,
            8,
            bar,
        );
    }
}
