use std::marker::PhantomData;
use std::ops::Deref;

use {Rational, format};
use super::codec::Codec;
use ffi::*;

pub struct Video<'a> {
	codec: &'a Codec<'a>,
}

impl<'a> Video<'a> {
	pub unsafe fn new<'b>(codec: &'b Codec) -> Video<'b> {
		Video {
			codec: codec,
		}
	}
}

impl<'a> Video<'a> {
	pub fn rates(&self) -> Option<RateIter> {
		unsafe {
			if (*self.codec.as_ptr()).supported_framerates.is_null() {
				None
			}
			else {
				Some(RateIter::new((*self.codec.as_ptr()).supported_framerates))
			}
		}
	}

	pub fn formats(&self) -> Option<FormatIter> {
		unsafe {
			if (*self.codec.as_ptr()).pix_fmts.is_null() {
				None
			}
			else {
				Some(FormatIter::new((*self.codec.as_ptr()).pix_fmts))
			}
		}
	}
}

impl<'a> Deref for Video<'a> {
	type Target = Codec<'a>;

	fn deref(&self) -> &Self::Target {
		self.codec
	}
}

pub struct RateIter<'a> {
	ptr: *const AVRational,

	_marker: PhantomData<&'a ()>,
}

impl<'a> RateIter<'a> {
	pub fn new(ptr: *const AVRational) -> Self {
		RateIter { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Iterator for RateIter<'a> {
	type Item = Rational;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if (*self.ptr).num == 0 && (*self.ptr).den == 0 {
				return None;
			}

			let rate = (*self.ptr).into();
			self.ptr = self.ptr.offset(1);

			Some(rate)
		}
	}
}

pub struct FormatIter<'a> {
	ptr: *const AVPixelFormat,

	_marker: PhantomData<&'a ()>,
}

impl<'a> FormatIter<'a> {
	pub fn new(ptr: *const AVPixelFormat) -> Self {
		FormatIter { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Iterator for FormatIter<'a> {
	type Item = format::Pixel;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if *self.ptr == AVPixelFormat::AV_PIX_FMT_NONE {
				return None;
			}

			let format = (*self.ptr).into();
			self.ptr   = self.ptr.offset(1);

			Some(format)
		}
	}
}
