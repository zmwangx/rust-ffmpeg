use std::ops::Deref;
use std::mem;

use ffi::*;
use ::Rational;
use super::Stream;
use format::context::common::Context;

pub struct StreamMut<'a> {
	context: &'a mut Context,
	index:   usize,

	immutable: Stream<'a>,
}

impl<'a> StreamMut<'a> {
	pub unsafe fn wrap(context: &mut Context, index: usize) -> StreamMut {
		StreamMut {
			context: mem::transmute_copy(&context),
			index:   index,

			immutable: Stream::wrap(mem::transmute_copy(&context), index)
		}
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVStream {
		*(*self.context.as_mut_ptr()).streams.offset(self.index as isize)
	}
}

impl<'a> StreamMut<'a> {
	pub fn set_time_base<R: Into<Rational>>(&mut self, value: R) {
		unsafe {
			(*self.as_mut_ptr()).time_base = value.into().into();
		}
	}

	pub fn set_rate<R: Into<Rational>>(&mut self, value: R) {
		unsafe {
			av_stream_set_r_frame_rate(self.as_mut_ptr(), value.into().into());
		}
	}
}

impl<'a> Deref for StreamMut<'a> {
	type Target = Stream<'a>;

	fn deref(&self) -> &Self::Target {
		&self.immutable
	}
}
