use std::convert::TryInto;
use std::ffi::CStr;
use std::io::Error;

use libc::c_char;
use libc::c_int;
use log_crate::LevelFilter;

use util::log::Level;

// This is ugly, but va_list is not stabilized

#[cfg(all(target_arch = "x86_64", target_family = "windows"))]
pub type Args = sys::va_list;

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
pub type Args = *mut sys::__va_list_tag;

pub struct LogContext {
    context: usize,
    level: Level,
    fmt: *const c_char,
    args: Args,
}

impl LogContext {
    /// Formats the message
    #[inline]
    pub fn to_message(&self) -> Result<String, Error> {
        unsafe { vsprintf::vsprintf(self.fmt, self.args) }
    }

    /// The format string
    #[inline]
    pub fn format(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.fmt) }
    }

    /// The log level
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// The log context. Mostly the address of whatever component called the log function.
    #[inline]
    pub fn context(&self) -> usize {
        self.context
    }

    /// The log varargs.
    ///
    /// **Platform dependant**, use with caution.
    #[inline]
    pub unsafe fn args(&self) -> Args {
        self.args
    }
}

pub trait Callback {
    fn call(context: &LogContext);
}

unsafe extern "C" fn wrapped_callback<T: Callback>(context: *mut libc::c_void, level: c_int, fmt: *const c_char, args: Args) {
    let context = LogContext {
        context: context as usize,
        level: level.try_into().unwrap_or(Level::Info),
        fmt,
        args,
    };
    T::call(&context);
}

/// Sets the log callback
pub fn set_callback<C: Callback>() {
    unsafe {
        sys::av_log_set_callback(Some(wrapped_callback::<C>));
    }
}

/// Resets the log callback
pub fn reset_callback() {
    unsafe {
        sys::av_log_set_callback(None);
    }
}

/// Sets the logging callback
///
/// Logs using the log crate to the target 'ffmpeg'
pub fn set_logging_callback() {
    set_callback::<LoggingCallback>();
}

/// Logs using the log crate to the target 'ffmpeg'
pub struct LoggingCallback;

impl Callback for LoggingCallback {
    fn call(context: &LogContext) {
        if let Some(log_level) = LevelFilter::from(context.level()).to_level() {
            // Don't format when level is disabled
            if log::log_enabled!(log_level) {
                match context.to_message() {
                    Ok(message) => log::log!(target: "ffmpeg", log_level, "{}", message.trim()),
                    Err(e) =>
                        log::warn!(target: "ffmpeg", "failed to format ffmpeg log message: {:?}", e),
                }
            }
        }
    }
}
