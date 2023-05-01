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

unsafe extern "C" fn log_callback(
    _arg1: *mut c_void,
    level: c_int,
    fmt: *const c_char,
    list: va_list,
) {
    let mut buffer = vec![0u8; 256];

    let result = vsnprintf(
        buffer.as_mut_ptr() as *mut i8,
        (buffer.capacity() as usize).try_into().unwrap(),
        fmt,
        list,
    );

    if result >= 0 {
        let len = result as usize;
        if len > buffer.capacity() {
            buffer.reserve(len - buffer.capacity());
            let result = vsnprintf(
                buffer.as_mut_ptr() as *mut i8,
                buffer.capacity().try_into().unwrap(),
                fmt,
                list,
            );
            assert!(result >= 0);
        }
        unsafe { buffer.set_len(len) };
        let cstring = CString::from_vec_unchecked(buffer);
        let log_message = cstring.to_string_lossy().into_owned();
        println!("Level {}: {}", level, log_message);
    } else {
        eprintln!("Error formatting log message");
    }
}
