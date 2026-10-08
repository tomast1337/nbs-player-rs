//! Tiny scoped stack profiler.
//!
//! Usage:
//! ```ignore
//! let _p = profiler::scope("update");   // times until end of block
//! ```
//! Scopes nest; each unique call path (`frame;draw;notes`) gets its own stats.
//! Call `end_frame()` once per frame, `draw_overlay()` to show the tree, and
//! `dump_folded()` to write `profile.folded` (feed to `inferno-flamegraph`).
//! While disabled a scope costs one thread-local bool read.

use raylib::prelude::*;
use std::{cell::RefCell, collections::HashMap, time::Instant};

const EMA_ALPHA: f32 = 0.1;
const PEAK_DECAY: f32 = 0.98;
const FOLDED_PATH: &str = "profile.folded";

struct Stat {
    name: &'static str,
    parent: Option<usize>,
    depth: usize,
    // per-frame accumulators
    frame_ns: u64,
    frame_calls: u32,
    // smoothed results
    avg_ms: f32,
    peak_ms: f32,
    calls: u32,
    // lifetime self time, for folded output
    self_ns_total: u64,
}

struct Open {
    stat: usize,
    start: Instant,
    child_ns: u64,
}

#[derive(Default)]
struct Profiler {
    enabled: bool,
    stack: Vec<Open>,
    stats: Vec<Stat>,
    index: HashMap<(Option<usize>, &'static str), usize>,
}

thread_local! {
    static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::default());
}

/// Drop guard returned by [`scope`]; records elapsed time when dropped.
pub struct Scope {
    active: bool,
}

pub fn scope(name: &'static str) -> Scope {
    let active = PROFILER.with(|p| {
        let mut p = p.borrow_mut();
        if !p.enabled {
            return false;
        }
        let parent = p.stack.last().map(|o| o.stat);
        let depth = parent.map_or(0, |i| p.stats[i].depth + 1);
        let stat = match p.index.get(&(parent, name)) {
            Some(&i) => i,
            None => {
                let i = p.stats.len();
                p.stats.push(Stat {
                    name,
                    parent,
                    depth,
                    frame_ns: 0,
                    frame_calls: 0,
                    avg_ms: 0.0,
                    peak_ms: 0.0,
                    calls: 0,
                    self_ns_total: 0,
                });
                p.index.insert((parent, name), i);
                i
            }
        };
        p.stack.push(Open {
            stat,
            start: Instant::now(),
            child_ns: 0,
        });
        true
    });
    Scope { active }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        PROFILER.with(|p| {
            let mut p = p.borrow_mut();
            // Scope may have been opened before profiling was toggled off.
            let Some(open) = p.stack.pop() else { return };
            let dur = open.start.elapsed().as_nanos() as u64;
            if let Some(parent) = p.stack.last_mut() {
                parent.child_ns += dur;
            }
            let stat = &mut p.stats[open.stat];
            stat.frame_ns += dur;
            stat.frame_calls += 1;
            stat.self_ns_total += dur.saturating_sub(open.child_ns);
        });
    }
}

pub fn is_enabled() -> bool {
    PROFILER.with(|p| p.borrow().enabled)
}

pub fn toggle() {
    PROFILER.with(|p| {
        let mut p = p.borrow_mut();
        p.enabled = !p.enabled;
        if !p.enabled {
            p.stack.clear();
        }
    });
}

/// Fold this frame's accumulators into the smoothed stats. Call once per frame
/// with no scopes open.
pub fn end_frame() {
    PROFILER.with(|p| {
        let mut p = p.borrow_mut();
        if !p.enabled {
            return;
        }
        for s in p.stats.iter_mut() {
            let ms = s.frame_ns as f32 / 1_000_000.0;
            s.avg_ms = if s.avg_ms == 0.0 {
                ms
            } else {
                s.avg_ms + EMA_ALPHA * (ms - s.avg_ms)
            };
            s.peak_ms = ms.max(s.peak_ms * PEAK_DECAY);
            s.calls = s.frame_calls;
            s.frame_ns = 0;
            s.frame_calls = 0;
        }
    });
}

/// Log the smoothed call tree (same data as the overlay).
pub fn log_report() {
    PROFILER.with(|p| {
        let p = p.borrow();
        for (i, s) in p.stats.iter().enumerate() {
            let mut path = vec![s.name];
            let mut cur = p.stats[i].parent;
            while let Some(pi) = cur {
                path.push(p.stats[pi].name);
                cur = p.stats[pi].parent;
            }
            path.reverse();
            log::info!(
                "{:<40} avg {:7.3} ms  peak {:7.3} ms  x{}",
                path.join(";"),
                s.avg_ms,
                s.peak_ms,
                s.calls
            );
        }
    });
}

/// F3 toggles profiling + overlay, F4 writes the folded-stack file.
pub fn handle_input(rl: &RaylibHandle) {
    if rl.is_key_pressed(KeyboardKey::KEY_F3) {
        toggle();
        log::info!("profiler {}", if is_enabled() { "on" } else { "off" });
    }
    if rl.is_key_pressed(KeyboardKey::KEY_F4) {
        dump_folded();
    }
}

/// Write `a;b;c <self-microseconds>` lines. Render with
/// `inferno-flamegraph < profile.folded > flame.svg`.
pub fn dump_folded() {
    let out = PROFILER.with(|p| {
        let p = p.borrow();
        let mut lines = String::new();
        for (i, s) in p.stats.iter().enumerate() {
            let mut path = vec![s.name];
            let mut cur = p.stats[i].parent;
            while let Some(pi) = cur {
                path.push(p.stats[pi].name);
                cur = p.stats[pi].parent;
            }
            path.reverse();
            lines.push_str(&format!("{} {}\n", path.join(";"), s.self_ns_total / 1000));
        }
        lines
    });
    match std::fs::write(FOLDED_PATH, out) {
        Ok(_) => log::info!("wrote {}", FOLDED_PATH),
        Err(err) => log::error!("failed to write {}: {}", FOLDED_PATH, err),
    }
}

/// Draw the call tree (avg / peak ms, calls, bar) at the top-right.
pub fn draw_overlay(d: &mut RaylibDrawHandle<'_>, window_width: f32, frame_time: f32) {
    const ROW_H: i32 = 14;
    const PANEL_W: i32 = 360;
    const BAR_W: f32 = 70.0;
    // Bars are scaled so a full bar is one 60 fps frame budget.
    const BUDGET_MS: f32 = 16.67;

    PROFILER.with(|p| {
        let p = p.borrow();
        if !p.enabled {
            return;
        }

        // Pre-order walk: children listed under their parent, in first-seen order.
        let mut order: Vec<usize> = Vec::with_capacity(p.stats.len());
        fn walk(stats: &[Stat], parent: Option<usize>, out: &mut Vec<usize>) {
            for (i, s) in stats.iter().enumerate() {
                if s.parent == parent {
                    out.push(i);
                    walk(stats, Some(i), out);
                }
            }
        }
        walk(&p.stats, None, &mut order);

        let x = window_width as i32 - PANEL_W - 10;
        let y = 30;
        let h = (order.len() as i32 + 2) * ROW_H + 8;
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

        for (row, &i) in order.iter().enumerate() {
            let s = &p.stats[i];
            let ry = y + (row as i32 + 1) * ROW_H + 4;
            let label = format!("{}{}", "  ".repeat(s.depth), s.name);
            d.draw_text(&label, x, ry, 10, Color::LIGHTGRAY);
            d.draw_text(
                &format!("{:6.3} / {:6.3}ms x{}", s.avg_ms, s.peak_ms, s.calls),
                x + 130,
                ry,
                10,
                Color::WHITE,
            );
            let frac = (s.avg_ms / BUDGET_MS).clamp(0.0, 1.0);
            let color = if frac > 0.5 {
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
                color,
            );
        }
    });
}
