use std::error;
use std::fmt;
use std::cell::RefCell;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use libc::c_int;
use ffi::*;

pub struct Error {
	code: c_int,
	desc: RefCell<Option<[i8; AV_ERROR_MAX_STRING_SIZE as usize]>>,
}

impl Error {
	pub fn new(code: c_int) -> Self {
		Error { code: code, desc: RefCell::new(None) }
	}

	pub fn bug() -> Self {
		Self::new(AVERROR_BUG)
	}
}

unsafe impl Send for Error { }

impl Clone for Error {
	fn clone(&self) -> Self {
		if let Some(old) = *self.desc.borrow() {
			Error {
				code: self.code,
				desc: RefCell::new(Some(old)),
			}
		}
		else {
			Error {
				code: self.code,
				desc: RefCell::new(None),
			}
		}
	}
}

impl From<c_int> for Error {
	fn from(value: c_int) -> Error {
		Error::new(value)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.write_str(error::Error::description(self))
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		try!(f.write_str("ffmpeg::Error("));
		try!(f.write_str(&format!("{}: ", AVUNERROR(self.code))));
		try!(fmt::Display::fmt(self, f));
		f.write_str(")")
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		unsafe {
			let mut desc = self.desc.borrow_mut();

			if let None = *desc {
				let mut buf = [0i8; AV_ERROR_MAX_STRING_SIZE as usize];
				av_strerror(self.code, buf.as_mut_ptr(), AV_ERROR_MAX_STRING_SIZE);

				*desc = Some(buf);
			}

			from_utf8_unchecked(CStr::from_ptr(desc.unwrap().as_ptr()).to_bytes())
		}
	}
}
