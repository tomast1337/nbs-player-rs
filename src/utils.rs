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

pub fn load_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    let path = std::ffi::CString::new(file_path).expect("CString failed");
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY, 0o666) };
    // Check for errors
    if fd < 0 {
        // Handle the error
        Err(std::io::Error::last_os_error())
    } else {
        println!(
            "[RUST + WASM] File opened successfully with file descriptor: {}",
            fd
        );

        // Read file content into a buffer
        let mut buffer = Vec::new();

        let temp_buffer = [0u8; 1024];
        loop {
            let bytes_read = unsafe {
                libc::read(
                    fd,
                    temp_buffer.as_ptr() as *mut libc::c_void,
                    temp_buffer.len(),
                )
            };
            if bytes_read < 0 {
                // Handle the error
                return Err(std::io::Error::last_os_error());
            } else if bytes_read == 0 {
                // End of file
                break;
            } else {
                buffer.extend_from_slice(&temp_buffer[..bytes_read as usize]);
            }
        }

        Ok(buffer)
    }
}

/*
pub fn logger_callback(level: raylib::ffi::TraceLogLevel, text: &str) {
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
*/
