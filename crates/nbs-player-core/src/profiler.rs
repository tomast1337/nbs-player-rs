//! Tiny scoped stack profiler.
//!
//! Usage:
//! ```ignore
//! let _p = profiler::scope("update");   // times until end of block
//! ```
//! Scopes nest; each unique call path (`frame;draw;notes`) gets its own stats.
//! Call `end_frame()` once per frame, show `rows()` however the frontend likes, and use
//! `dump_folded()` to write `profile.folded` (feed to `inferno-flamegraph`).
//! While disabled a scope costs one thread-local bool read.
//!
//! Time comes from a clock function returning nanoseconds. Native targets default to
//! `std::time::Instant`; `wasm32-unknown-unknown` has no std clock (it panics), so a web
//! frontend must call [`set_clock`] with its own (`performance.now`, `miniquad::date::now`).

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

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

/// Monotonic nanoseconds from an arbitrary origin.
pub type Clock = fn() -> u64;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn default_clock() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

// No std clock here; scopes read as zero until the frontend installs one.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn default_clock() -> u64 {
    0
}

struct Open {
    stat: usize,
    start: u64,
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
    static CLOCK: Cell<Clock> = const { Cell::new(default_clock) };
}

/// Replace the time source (needed on `wasm32-unknown-unknown`).
pub fn set_clock(clock: Clock) {
    CLOCK.with(|c| c.set(clock));
}

fn now_ns() -> u64 {
    CLOCK.with(|c| c.get())()
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
            start: now_ns(),
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
            let dur = now_ns().saturating_sub(open.start);
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
    write_folded(&out);
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn write_folded(out: &str) {
    match std::fs::write(FOLDED_PATH, out) {
        Ok(_) => log::info!("wrote {}", FOLDED_PATH),
        Err(err) => log::error!("failed to write {}: {}", FOLDED_PATH, err),
    }
}

// No filesystem in the browser: print the folded stacks to the console instead.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn write_folded(out: &str) {
    log::info!("{FOLDED_PATH}:\n{out}");
}

/// One line of the call tree, in pre-order (children directly under their parent).
#[derive(Debug, Clone)]
pub struct Row {
    pub depth: usize,
    pub name: &'static str,
    pub avg_ms: f32,
    pub peak_ms: f32,
    pub calls: u32,
}

/// Snapshot of the smoothed call tree. Empty while the profiler is disabled.
pub fn rows() -> Vec<Row> {
    PROFILER.with(|p| {
        let p = p.borrow();
        if !p.enabled {
            return Vec::new();
        }
        fn walk(stats: &[Stat], parent: Option<usize>, out: &mut Vec<Row>) {
            for (i, s) in stats.iter().enumerate() {
                if s.parent == parent {
                    out.push(Row {
                        depth: s.depth,
                        name: s.name,
                        avg_ms: s.avg_ms,
                        peak_ms: s.peak_ms,
                        calls: s.calls,
                    });
                    walk(stats, Some(i), out);
                }
            }
        }
        let mut out = Vec::with_capacity(p.stats.len());
        walk(&p.stats, None, &mut out);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NOW: AtomicU64 = AtomicU64::new(0);
    fn fake() -> u64 {
        NOW.load(Ordering::Relaxed)
    }

    #[test]
    fn custom_clock_drives_scopes() {
        set_clock(fake);
        toggle();
        {
            let _outer = scope("outer");
            NOW.store(2_000_000, Ordering::Relaxed);
            {
                let _inner = scope("inner");
                NOW.store(5_000_000, Ordering::Relaxed);
            }
            NOW.store(6_000_000, Ordering::Relaxed);
        }
        end_frame();
        let rows = rows();
        toggle();
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].name, rows[0].avg_ms), ("outer", 6.0));
        assert_eq!((rows[1].name, rows[1].avg_ms, rows[1].depth), ("inner", 3.0, 1));
    }
}
