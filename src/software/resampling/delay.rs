use libc::int64_t;
use ffi::*;
use super::Context;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Delay {
	pub seconds:      i64,
	pub milliseconds: i64,
	pub input:        i64,
	pub output:       i64,
}

impl Delay {
	pub fn from(context: &Context) -> Self {
		unsafe {
			Delay {
				seconds:      swr_get_delay(context.as_ptr() as *mut _, 1),
				milliseconds: swr_get_delay(context.as_ptr() as *mut _, 1000),
				input:        swr_get_delay(context.as_ptr() as *mut _, context.input().rate as int64_t),
				output:       swr_get_delay(context.as_ptr() as *mut _, context.output().rate as int64_t),
			}
		}
	}
}
