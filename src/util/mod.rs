#[macro_use]
pub mod dictionary;
pub mod error;
pub mod rational;
pub mod media;
pub mod picture;
pub mod color;
pub mod format;
pub mod frame;
pub mod chroma;
pub mod time;
pub mod channel_layout;
pub mod option;
pub mod range;
pub mod mathematics;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

#[inline(always)]
pub fn version() -> u32 {
	unsafe {
		avutil_version()
	}
}

#[inline(always)]
pub fn configuration() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avutil_configuration()).to_bytes())
	}
}

#[inline(always)]
pub fn license() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avutil_license()).to_bytes())
	}
}
