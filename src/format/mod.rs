pub use ::util::format::Sample;
pub use ::util::format::Pixel;

pub mod stream;

pub mod context;
pub use self::context::{Context, open, open_with, open_as, open_as_with, dump};

pub mod format;
pub use self::format::{Input, Output, list};

pub mod network;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;
use ::Format;

pub fn register_all() {
	unsafe {
		av_register_all();
	}
}

pub fn register(format: &Format) {
	match format {
		&Format::Input(ref format) => unsafe {
			av_register_input_format(format.ptr);
		},

		&Format::Output(ref format) => unsafe {
			av_register_output_format(format.ptr);
		}
	}
}

pub fn version() -> u32 {
	unsafe {
		avformat_version()
	}
}

pub fn configuration() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avformat_configuration()).to_bytes())
	}
}

pub fn license() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avformat_license()).to_bytes())
	}
}
