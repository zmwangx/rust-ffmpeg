pub use ::util::format::{sample, Sample};
pub use ::util::format::{pixel, Pixel};

pub mod stream;

pub mod context;
pub use self::context::Context;

pub mod format;
pub use self::format::{Input, Output, list};

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
fn from_path<T: AsRef<Path>>(path: &T) -> CString {
	CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap()
}

pub fn open_input<T: AsRef<Path>>(path: &T) -> Result<Context, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     status = avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), ptr::null_mut());

		match status {
			0 => {
				let ctx = Context::input(ps);

				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(ctx),
					e => Err(Error::from(e))
				}
			},

			e => Err(Error::from(e))
		}
	}
}

pub fn open_input_with<T: AsRef<Path>>(path: &T, options: Dictionary) -> Result<Context, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let mut opts   = options.take();
		let     status = avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), &mut opts);

		Dictionary::own(opts);

		match status {
			0 => {
				let ctx = Context::input(ps);

				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(ctx),
					e => Err(Error::from(e))
				}
			},

			e => Err(Error::from(e))
		}
	}
}

pub fn open_input_as<T: AsRef<Path>>(path: &T, format: &Format) -> Result<Context, Error> {
	if let &Format::Input(ref format) = format {
		unsafe {
			let mut ps     = ptr::null_mut();
			let     path   = from_path(path);
			let     status = avformat_open_input(&mut ps, path.as_ptr(), format.as_ptr(), ptr::null_mut());

			match status {
				0 => {
					let ctx = Context::input(ps);

					match avformat_find_stream_info(ps, ptr::null_mut()) {
						0 => Ok(ctx),
						e => Err(Error::from(e))
					}
				},

				e => Err(Error::from(e))
			}
		}
	}
	else {
		Err(Error::Bug)
	}
}

pub fn open_input_as_with<T: AsRef<Path>>(path: &T, format: &Format, options: Dictionary) -> Result<Context, Error> {
	if let &Format::Input(ref format) = format {
		unsafe {
			let mut ps     = ptr::null_mut();
			let     path   = from_path(path);
			let mut opts   = options.take();
			let     status = avformat_open_input(&mut ps, path.as_ptr(), format.as_ptr(), &mut opts);

			Dictionary::own(opts);

			match status {
				0 => {
					let ctx = Context::input(ps);

					match avformat_find_stream_info(ps, ptr::null_mut()) {
						0 => Ok(ctx),
						e => Err(Error::from(e))
					}
				},

				e => Err(Error::from(e))
			}
		}
	}
	else {
		Err(Error::Bug)
	}
}

pub fn open_output<T: AsRef<Path>>(path: &T) -> Result<Context, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     status = avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr());

		match status {
			0 => {
				match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
					0 => Ok(Context::output(ps)),
					e => Err(Error::from(e)),
				}
			},
			e => Err(Error::from(e))
		}
	}
}

pub fn open_output_as<T: AsRef<Path>>(path: &T, format: &Format) -> Result<Context, Error> {
    if let &Format::Output(ref format) = format {
	    unsafe {
		    let mut ps     = ptr::null_mut();
		    let     path   = from_path(path);
		    let     status = avformat_alloc_output_context2(&mut ps, format.as_ptr(), ptr::null(), path.as_ptr());

		    match status {
				0 => {
					match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
						0 => Ok(Context::output(ps)),
						e => Err(Error::from(e)),
					}
				},
			    e => Err(Error::from(e))
		    }
	    }
    }
    else {
        Err(Error::Bug)
    }
}

pub fn open_output_as_string<T: AsRef<Path>>(path: &T, format: &str) -> Result<Context, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     format = CString::new(format).unwrap();
		let     status = avformat_alloc_output_context2(&mut ps, ptr::null_mut(), format.as_ptr(), path.as_ptr());

		match status {
				0 => {
					match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
						0 => Ok(Context::output(ps)),
						e => Err(Error::from(e)),
					}
				},
			e => Err(Error::from(e))
		}
	}
}

pub fn dump(ctx: &Context, index: i32, url: Option<&str>) {
	let url = url.map(|u| CString::new(u).unwrap());

	unsafe {
		av_dump_format(ctx.as_ptr(), index,
			url.map(|u| u.as_ptr()).unwrap_or(ptr::null()),
			if ctx.is_input() { 0 } else { 1 });
	}
}
