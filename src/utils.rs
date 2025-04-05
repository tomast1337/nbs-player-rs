use raylib::color::Color;

/// Formats a time in seconds to a string in the format "mm:ss".
#[inline]
pub fn time_formatter(time: f32) -> String {
    let minutes = (time / 60.0).floor() as u32;
    let seconds = (time % 60.0) as u32;
    format!("{:0>2}:{:0>2}", minutes, seconds)
}

/// Linear interpolation function for smooth animation
#[inline]
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start.mul_add(1.0 - t, end * t)
}

pub fn load_file(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    let path = std::ffi::CString::new(file_path).expect("CString failed");
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY, 0o666) };
    // Check for errors
    if fd < 0 {
        // Handle the error
        Err(std::io::Error::last_os_error())
    } else {
        println!("File opened successfully with file descriptor: {}", fd);

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

pub fn string_to_c_char(s: String) -> *const std::ffi::c_char {
    // Create a CString, which will add a null terminator
    let c_string = std::ffi::CString::new(s).expect("CString::new failed");

    // Convert into a raw pointer and leak it (prevent Rust from freeing the memory)
    let ptr = c_string.into_raw();

    // into_raw() gives us a *mut c_char, but we need *const
    ptr as *const std::ffi::c_char
}

pub fn fast_pow2(x: f32) -> f32 {
    let x0 = x.floor();
    let x1 = x - x0;

    // Handle overflow and underflow
    if x0 >= 32.0 {
        return f32::INFINITY; // 2^x is too large for f32
    } else if x0 <= -32.0 {
        return 0.0; // 2^x is too small for f32
    }

    // Calculate 2^x1 using a polynomial approximation
    let p = 1.0 + x1 * (0.693147 + x1 * (0.241586 + x1 * 0.052043));

    // Calculate 2^x0 using bit shifting (only for positive x0)
    if x0 >= 0.0 {
        p * (1 << x0 as i32) as f32
    } else {
        p / (1 << (-x0 as i32)) as f32
    }
}

pub fn blend_colors(base: Color, tints: &[(Color, f32)]) -> Color {
    let mut r = base.r as f32;
    let mut g = base.g as f32;
    let mut b = base.b as f32;
    let mut a = base.a as f32;

    let n = tints.len().max(1) as f32;
    let inv_n = 1.0 / n;

    for i in 0..tints.len() {
        let (color, weight) = &tints[i];
        let position_factor = (n - i as f32) * inv_n;
        let effective_weight = weight * position_factor;
        let inv_weight = 1.0 - effective_weight;

        r = r * inv_weight + color.r as f32 * effective_weight;
        g = g * inv_weight + color.g as f32 * effective_weight;
        b = b * inv_weight + color.b as f32 * effective_weight;
        a = a * inv_weight + color.a as f32 * effective_weight;
    }

    Color::new(
        r.round() as u8,
        g.round() as u8,
        b.round() as u8,
        a.round() as u8,
    )
}
