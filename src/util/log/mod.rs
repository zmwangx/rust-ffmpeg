pub mod level;
pub use self::level::Level;
use std::ffi::CString;
pub mod flag;
pub use self::flag::Flags;

use ffi::*;
use std::convert::TryInto;

pub fn set_level(value: Level) {
    unsafe { av_log_set_level(value.into()) }
}

pub fn get_level() -> Result<Level, &'static str> {
    unsafe { av_log_get_level().try_into() }
}

pub fn set_flags(value: Flags) {
    unsafe { av_log_set_flags(value.bits()) }
}

pub fn get_flags() -> Flags {
    unsafe { Flags::from_bits_truncate(av_log_get_flags()) }
}

pub fn set_callback() {
    unsafe {
        av_log_set_callback(Some(log_callback));
    }
}

use std::os::raw::{c_char, c_int, c_void};

const INITIAL_BUFFER_SIZE: usize = 512;

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
type Arg = __builtin_va_list;

#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
type Arg = *mut __va_list_tag;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
type Arg = va_list;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
type Arg = *mut __va_list_tag;

#[cfg(target_os = "windows")]
type Arg = va_list;

unsafe extern "C" fn log_callback(_arg1: *mut c_void, level: c_int, fmt: *const c_char, list: Arg) {
    let mut buffer = Vec::with_capacity(INITIAL_BUFFER_SIZE);
    buffer.resize(INITIAL_BUFFER_SIZE, 0 as c_char);

    let result = vsnprintf(buffer.as_mut_ptr(), buffer.len() as u64, fmt, list);

    if result >= 0 {
        let len = result as usize;
        if len > buffer.capacity() {
            buffer.reserve(len - buffer.capacity());
            let result = vsnprintf(buffer.as_mut_ptr(), buffer.len() as u64, fmt, list);
            assert!(result >= 0);
        }
        unsafe { buffer.set_len(len) };
        let cstring = CString::from_vec_unchecked(buffer.iter().map(|&x| x as u8).collect());
        let log_message = cstring.to_string_lossy().into_owned();
        println!("Level {}: {}", level, log_message);
    } else {
        eprintln!("Error formatting log message");
    }
}
