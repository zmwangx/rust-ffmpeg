use ffi::*;
use ::Error;

pub fn current() -> i64 {
	unsafe {
		av_gettime() as i64
	}
}

pub fn relative() -> i64 {
	unsafe {
		av_gettime_relative() as i64
	}
}

pub fn is_monotonic() -> bool {
	unsafe {
		av_gettime_relative_is_monotonic() != 0
	}
}

pub fn sleep(usec: u32) -> Result<(), Error> {
	unsafe {
		match av_usleep(usec) {
			0 => Ok(()),
			e => Err(Error::from(e))
		}
	}
}
