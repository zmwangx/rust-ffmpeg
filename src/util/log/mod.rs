pub mod level;
pub use self::level::Level;

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
