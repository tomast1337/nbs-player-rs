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
