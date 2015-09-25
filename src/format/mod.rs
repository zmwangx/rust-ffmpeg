pub use ::util::format::{sample, Sample};
pub use ::util::format::{pixel, Pixel};

pub mod stream;

pub mod context;
pub use self::context::Context;

pub mod format;
pub use self::format::{Input, Output, list};
pub use self::format::{flag, Flags};

pub mod network;

use std::ptr;
use std::path::Path;
use std::ffi::{CString, CStr};
use std::str::from_utf8_unchecked;

use ffi::*;
use ::{Error, Format, Dictionary};

pub fn register_all() {
	unsafe {
		av_register_all();
	}
}

pub fn register(format: &Format) {
	match format {
		&Format::Input(ref format) => unsafe {
			av_register_input_format(format.as_ptr());
		},

		&Format::Output(ref format) => unsafe {
			av_register_output_format(format.as_ptr());
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

// XXX: use to_cstring when stable
fn from_path<P: AsRef<Path>>(path: &P) -> CString {
	CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap()
}

// NOTE: this will be better with specialization or anonymous return types
pub fn open<P: AsRef<Path>>(path: &P, format: &Format) -> Result<Context, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);

		match format {
			&Format::Input(ref format) => {
				match avformat_open_input(&mut ps, path.as_ptr(), format.as_ptr(), ptr::null_mut()) {
					0 => {
						match avformat_find_stream_info(ps, ptr::null_mut()) {
							0 => Ok(Context::Input(context::Input::wrap(ps))),
							e => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}

			&Format::Output(ref format) => {
				match avformat_alloc_output_context2(&mut ps, format.as_ptr(), ptr::null(), path.as_ptr()) {
					0 => {
						match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
							0 => Ok(Context::Output(context::Output::wrap(ps))),
							e => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}
		}
	}
}

pub fn open_with<P: AsRef<Path>>(path: &P, format: &Format, options: Dictionary) -> Result<Context, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);
		let mut opts = options.disown();

		match format {
			&Format::Input(ref format) => {
				let res = avformat_open_input(&mut ps, path.as_ptr(), format.as_ptr(), &mut opts);

				Dictionary::own(opts);

				match res {
					0 => {
						match avformat_find_stream_info(ps, ptr::null_mut()) {
							0 => Ok(Context::Input(context::Input::wrap(ps))),
							e => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}

			&Format::Output(ref format) => {
				match avformat_alloc_output_context2(&mut ps, format.as_ptr(), ptr::null(), path.as_ptr()) {
					0 => {
						match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
							0 => Ok(Context::Output(context::Output::wrap(ps))),
							e => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}
		}
	}
}

pub fn input<P: AsRef<Path>>(path: &P) -> Result<context::Input, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);

		match avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), ptr::null_mut()) {
			0 => {
				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(context::Input::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn input_with<P: AsRef<Path>>(path: &P, options: Dictionary) -> Result<context::Input, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);
		let mut opts = options.disown();
		let     res  = avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), &mut opts);

		Dictionary::own(opts);

		match res {
			0 => {
				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(context::Input::wrap(ps)),
					e => Err(Error::from(e))
				}
			}
			
			e => Err(Error::from(e))
		}
	}
}

pub fn output<P: AsRef<Path>>(path: &P) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
			0 => {
				match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output_with<P: AsRef<Path>>(path: &P, options: Dictionary) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let mut opts   = options.disown();

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
			0 => {
				let res = avio_open2(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE, ptr::null(), &mut opts,);

				Dictionary::own(opts);

				match res {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output_as<P: AsRef<Path>>(path: &P, format: &str) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     format = CString::new(format).unwrap();

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), format.as_ptr(), path.as_ptr()) {
			0 => {
				match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output_as_with<P: AsRef<Path>>(path: &P, format: &str, options: Dictionary) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     format = CString::new(format).unwrap();
		let mut opts   = options.disown();

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), format.as_ptr(), path.as_ptr()) {
			0 => {
				let res = avio_open2(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE, ptr::null(), &mut opts,);

				Dictionary::own(opts);

				match res {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}
