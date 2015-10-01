use libc::c_int;
use ffi::*;
use ::codec::{self, packet};
use ::{Rational, Discard};
use super::Disposition;
use format::context::common::Context;

pub struct Stream<'a> {
	context: &'a Context,
	index:   usize,
}

impl<'a> Stream<'a> {
	pub unsafe fn wrap(context: &Context, index: usize) -> Stream {
		Stream { context: context, index: index }
	}

	pub unsafe fn as_ptr(&self) -> *const AVStream {
		*(*self.context.as_ptr()).streams.offset(self.index as isize)
	}
}

impl<'a> Stream<'a> {
	pub fn codec(&self) -> codec::Context {
		unsafe {
			codec::Context::wrap((*self.as_ptr()).codec, Some(self.context.destructor()))
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
		SideDataIter::new(self)
	}

	pub fn rate(&self) -> Rational {
		unsafe {
			Rational::from(av_stream_get_r_frame_rate(self.as_ptr()))
		}
	}
}

impl<'a> PartialEq for Stream<'a> {
	fn eq(&self, other: &Self) -> bool {
		unsafe {
			self.as_ptr() == other.as_ptr()
		}
	}
}

impl<'a> Eq for Stream<'a> { }

pub struct SideDataIter<'a> {
	stream: &'a Stream<'a>,
	current: c_int,
}

impl<'a> SideDataIter<'a> {
	pub fn new<'sd, 's: 'sd>(stream: &'s Stream) -> SideDataIter<'sd> {
		SideDataIter { stream: stream, current: 0 }
	}
}

impl<'a> Iterator for SideDataIter<'a> {
	type Item = packet::SideData<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.current >= (*self.stream.as_ptr()).nb_side_data {
				return None;
			}

			self.current += 1;

			Some(packet::SideData::wrap(
				(*self.stream.as_ptr()).side_data.offset((self.current - 1) as isize)))
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			let length = (*self.stream.as_ptr()).nb_side_data as usize;

			(length - self.current as usize, Some(length - self.current as usize))
		}
	}
}

impl<'a> ExactSizeIterator for SideDataIter<'a> { }
