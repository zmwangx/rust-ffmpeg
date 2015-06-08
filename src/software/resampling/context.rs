use std::ptr;

use libc::{c_int, int64_t};
use ffi::*;
use ::util::format;
use ::{Error, ChannelLayout, frame};
use super::Delay;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Definition {
	pub format:         format::Sample,
	pub channel_layout: ChannelLayout,
	pub rate:           u32,
}

pub struct Context {
	ptr: *mut SwrContext,

	input:  Definition,
	output: Definition,
}

impl Context {
	pub unsafe fn as_ptr(&self) -> *const SwrContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut SwrContext {
		self.ptr
	}
}

impl Context {
	pub fn get(src_format: format::Sample, src_channel_layout: ChannelLayout, src_rate: u32,
	           dst_format: format::Sample, dst_channel_layout: ChannelLayout, dst_rate: u32) -> Result<Self, Error> {
		unsafe {
			let ptr = swr_alloc_set_opts(ptr::null_mut(),
				dst_channel_layout.bits() as int64_t, dst_format.into(), dst_rate as c_int,
				src_channel_layout.bits() as int64_t, src_format.into(), src_rate as c_int,
				0, ptr::null_mut());

			if ptr != ptr::null_mut() {
				match swr_init(ptr) {
					e if e < 0 =>
						Err(Error::from(e)),

					_ =>
						Ok(Context {
							ptr: ptr,

							input: Definition {
								format:         src_format,
								channel_layout: src_channel_layout,
								rate:           src_rate,
							},

							output: Definition {
								format:         dst_format,
								channel_layout: dst_channel_layout,
								rate:           dst_rate,
							}
						})
				}
			}
			else {
				Err(Error::InvalidData)
			}
		}
	}

	pub fn input(&self) -> &Definition {
		&self.input
	}

	pub fn output(&self) -> &Definition {
		&self.output
	}

	pub fn delay(&self) -> Option<Delay> {
		unsafe {
			match swr_get_delay(self.as_ptr(), 1) {
				0 => None,
				_ => Some(Delay::from(self))
			}
		}
	}

	pub fn run(&mut self, input: &frame::Audio, output: &mut frame::Audio) -> Result<Option<Delay>, Error> {
		output.set_rate(self.output.rate);

		unsafe {
			if output.is_empty() {
				output.alloc(self.output.format, input.samples(), self.output.channel_layout);
			}

			match swr_convert_frame(self.as_mut_ptr(), output.as_mut_ptr(), input.as_ptr()) {
				0 =>
					Ok(self.delay()),

				e =>
					Err(Error::from(e))
			}
		}
	}

	pub fn next(&mut self, output: &mut frame::Audio) -> Result<Option<Delay>, Error> {
		output.set_rate(self.output.rate);

		unsafe {
			match swr_convert_frame(self.as_mut_ptr(), output.as_mut_ptr(), ptr::null()) {
				0 =>
					Ok(self.delay()),

				e =>
					Err(Error::from(e))
			}
		}
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			swr_free(&mut self.as_mut_ptr());
		}
	}
}
