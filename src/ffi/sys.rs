use std::os::raw::c_char;
use std::ffi::{CString, CStr};
use anyhow::{anyhow, Result};



extern "C" {
    fn sky_get_cwd(buf: *mut c_char, buf_size: i32) -> i32;
    fn sky_check_file_access(path: *const c_char) -> i32;
}

/// Get the current directory
pub fn get_cwd() -> Result<String> {
    let buf_size = 1024;
    let mut buf = vec![0; buf_size as usize];
    let result = unsafe { sky_get_cwd(buf.as_mut_ptr(), buf_size) };
    if result != 0 {
        return Err(anyhow!("Failed to get current directory (error code: {})", result));
    }
    let c_str = unsafe { CStr::from_ptr(buf.as_ptr()) };
    Ok(c_str.to_str()?.to_string())
}

/// Check if a file is readable and writable
pub fn check_file_access(path: &str) -> Result<()> {
    let c_path = CString::new(path)?;
    let result = unsafe { sky_check_file_access(c_path.as_ptr()) };
    if result != 0 {
        return Err(anyhow!("File not accessible ({}), error code: {}", path, result));
    }
    Ok(())
}