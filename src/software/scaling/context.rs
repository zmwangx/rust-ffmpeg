use std::ptr;

use libc::{c_int};
use ffi::*;
use ::{Error, Picture};
use ::util::format;
use super::Flags;

pub struct Context {
	pub ptr: *mut SwsContext,

	input:  Definition,
	output: Definition,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Definition {
	pub format: format::Pixel,
	pub width:  u32,
	pub height: u32,
}

impl Context {
	pub fn get(src_format: format::Pixel, src_w: u32, src_h: u32,
	           dst_format: format::Pixel, dst_w: u32, dst_h: u32,
	           flags: Flags) -> Result<Self, Error> {
		unsafe {
			let ptr = sws_getContext(src_w as c_int, src_h as c_int, src_format.into(),
			                         dst_w as c_int, dst_h as c_int, dst_format.into(),
			                         flags.bits(),
			                         ptr::null_mut(), ptr::null_mut(), ptr::null_mut());

			if ptr != ptr::null_mut() {
				Ok(Context {
					ptr: ptr,

					input: Definition {
						format: src_format,
						width:  src_w,
						height: src_h,
					},

					output: Definition {
						format: dst_format,
						width:  dst_w,
						height: dst_h,
					},
				})
			}
			else {
				Err(Error::InvalidData)
			}
		}
	}

	pub fn cached(&mut self,
	              src_format: format::Pixel, src_w: u32, src_h: u32,
	              dst_format: format::Pixel, dst_w: u32, dst_h: u32,
	              flags: Flags) {
		self.input = Definition {
			format: src_format,
			width:  src_w,
			height: src_h,
		};

		self.output = Definition {
			format: dst_format,
			width:  dst_w,
			height: dst_h,
		};

		unsafe {
			self.ptr = sws_getCachedContext(self.ptr,
				src_w as c_int, src_h as c_int, src_format.into(),
				dst_w as c_int, dst_h as c_int, dst_format.into(),
				flags.bits(), ptr::null_mut(), ptr::null_mut(), ptr::null());
		}
	}

	pub fn input(&self) -> &Definition {
		&self.input
	}

	pub fn output(&self) -> &Definition {
		&self.output
	}

	pub fn run(&self, input: &Picture, output: &mut Picture) -> Result<(), Error> {
		if input.format() != self.input.format || input.width() != self.input.width || input.height() != self.input.height {
			return Err(Error::InputChanged);
		}

		if output.format() != self.output.format || output.width() != self.output.width || output.height() != self.output.height {
			return Err(Error::OutputChanged);
		}

		unsafe {
			sws_scale(self.ptr,
				(*input.ptr).data.as_ptr() as *const *const _, (*input.ptr).linesize.as_ptr() as *const _,
				0, self.output.height as c_int,
				(*output.ptr).data.as_ptr() as *mut *mut _, (*output.ptr).linesize.as_ptr() as *mut _);
		}

		Ok(())
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			sws_freeContext(self.ptr);
		}
	}
}
