use std::marker::PhantomData;

use libc::c_int;
use ffi::*;
use ::format;
use ::codec::{self, packet};
use ::{Rational, Discard};
use super::Disposition;

#[derive(Eq, PartialEq)]
pub struct Stream<'a> {
	ptr: *mut AVStream,

	_marker: PhantomData<&'a format::Context>,
}

impl<'a> Stream<'a> {
	pub unsafe fn wrap(ptr: *mut AVStream) -> Self {
		Stream { ptr: ptr, _marker: PhantomData }
	}

	pub unsafe fn as_ptr(&self) -> *const AVStream {
		self.ptr as *const _
	}
}

impl<'a> Stream<'a> {
	pub fn codec(&self) -> codec::Context {
		unsafe {
			codec::Context::wrap((*self.as_ptr()).codec)
		}
	}

	pub fn index(&self) -> usize {
		unsafe {
			(*self.as_ptr()).index as usize
		}
	}

	pub fn time_base(&self) -> Rational {
		unsafe {
			Rational::from((*self.as_ptr()).time_base)
		}
	}

	pub fn start_time(&self) -> i64 {
		unsafe {
			(*self.as_ptr()).start_time
		}
	}

	pub fn duration(&self) -> i64 {
		unsafe {
			(*self.as_ptr()).duration
		}
	}

	pub fn frames(&self) -> i64 {
		unsafe {
			(*self.as_ptr()).nb_frames
		}
	}

	pub fn disposition(&self) -> Disposition {
		unsafe {
			Disposition::from_bits_truncate((*self.as_ptr()).disposition)
		}
	}

	pub fn discard(&self) -> Discard {
		unsafe {
			Discard::from((*self.as_ptr()).discard)
		}
	}

	pub fn side_data(&self) -> SideDataIter {
		unsafe {
			SideDataIter::new(self.as_ptr())
		}
	}

	pub fn frame_rate(&self) -> Rational {
		unsafe {
			Rational::from(av_stream_get_r_frame_rate(self.as_ptr()))
		}
	}
}

pub struct SideDataIter<'a> {
	ptr: *const AVStream,
	cur: c_int,

	_marker: PhantomData<&'a Stream<'a>>,
}

impl<'a> SideDataIter<'a> {
	pub fn new(ptr: *const AVStream) -> Self {
		SideDataIter { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> Iterator for SideDataIter<'a> {
	type Item = packet::SideData<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).nb_side_data {
				None
			}
			else {
				self.cur += 1;
				Some(packet::SideData::wrap((*self.ptr).side_data.offset((self.cur - 1) as isize)))
			}
		}
	}
}
