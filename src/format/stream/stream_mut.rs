use std::ops::Deref;

use ffi::*;
use ::Rational;
use super::Stream;

#[derive(Eq, PartialEq)]
pub struct StreamMut<'a> {
	ptr: *mut AVStream,
	imm: Stream<'a>,
}

impl<'a> StreamMut<'a> {
	pub unsafe fn wrap(ptr: *mut AVStream) -> Self {
		StreamMut { ptr: ptr, imm: Stream::wrap(ptr) }
	}

	pub unsafe fn as_ptr(&self) -> *const AVStream {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVStream {
		self.ptr
	}
}

impl<'a> StreamMut<'a> {
	pub fn set_time_base<R: Into<Rational>>(&mut self, value: R) {
		unsafe {
			(*self.as_mut_ptr()).time_base = value.into().into();
		}
	}

	pub fn set_frame_rate<R: Into<Rational>>(&mut self, value: R) {
		unsafe {
			av_stream_set_r_frame_rate(self.as_mut_ptr(), value.into().into());
		}
	}
}

impl<'a> Deref for StreamMut<'a> {
	type Target = Stream<'a>;

	fn deref(&self) -> &Self::Target {
		&self.imm
	}
}
