use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Validates OpenTofu configuration and returns a JSON string with validation results
/// 
/// # Safety
/// - `input` must be a valid null-terminated C string
/// - Caller must free the returned string using `free_string`
#[no_mangle]
pub unsafe extern "C" fn validate_config(input: *const c_char) -> *mut c_char {
    // Convert C string to Rust string
    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("{\"error\":\"Invalid UTF-8 in input\"}").unwrap().into_raw(),
    };

    // Perform validation (example implementation)
    let result = match validate_config_internal(input_str) {
        Ok(valid) => {
            if valid {
                "{\"valid\":true}"
            } else {
                "{\"valid\":false,\"errors\":[\"Configuration invalid\"]}"
            }
        }
        Err(e) => format!("{{\"error\":\"{}\"}}", e),
    };

    // Convert result to C string
    CString::new(result).unwrap().into_raw()
}

/// Frees a string allocated by Rust
/// 
/// # Safety
/// - `ptr` must have been created by Rust's CString::into_raw()
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

// Internal validation function
fn validate_config_internal(config: &str) -> anyhow::Result<bool> {
    // Add your validation logic here
    // This is just an example implementation
    if config.contains("resource") {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_validate_config() {
        let input = CString::new(r#"{"resource": "aws_instance"}"#).unwrap();
        let result = unsafe { validate_config(input.as_ptr()) };
        let output = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert_eq!(output, r#"{"valid":true}"#);
        unsafe { free_string(result) };
    }
}
