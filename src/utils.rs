use raylib::ffi::TraceLogLevel;

/// Formats a time in seconds to a string in the format "mm:ss".
pub fn time_formatter(time: f32) -> String {
    let minutes = (time / 60.0).floor() as u32;
    let seconds = (time % 60.0) as u32;
    format!("{:0>2}:{:0>2}", minutes, seconds)
}

/// Linear interpolation function for smooth animation
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

pub fn logger_callback(level: TraceLogLevel, text: &str) {
    match level {
        TraceLogLevel::LOG_ALL => log::trace!("{}", text),
        TraceLogLevel::LOG_TRACE => log::trace!("{}", text),
        TraceLogLevel::LOG_DEBUG => log::debug!("{}", text),
        TraceLogLevel::LOG_INFO => log::info!("{}", text),
        TraceLogLevel::LOG_WARNING => log::warn!("{}", text),
        TraceLogLevel::LOG_ERROR => log::error!("{}", text),
        TraceLogLevel::LOG_FATAL => log::error!("{}", text),
        TraceLogLevel::LOG_NONE => {}
    }
}
