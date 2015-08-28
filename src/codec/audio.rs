use std::ops::Deref;
use std::marker::PhantomData;

use {ChannelLayout, format};
use super::codec::Codec;
use ffi::*;

pub struct Audio<'a> {
	codec: &'a Codec<'a>,
}

impl<'a> Audio<'a> {
	pub unsafe fn new<'b>(codec: &'b Codec) -> Audio<'b> {
		Audio {
			codec: codec,
		}
	}
}

impl<'a> Audio<'a> {
	pub fn rates(&self) -> RateIter {
		unsafe {
			RateIter::new((*self.codec.as_ptr()).supported_samplerates)
		}
	}

	pub fn formats(&self) -> FormatIter {
		unsafe {
			FormatIter::new((*self.codec.as_ptr()).sample_fmts)
		}
	}

	pub fn channel_layouts(&self) -> ChannelLayoutIter {
		unsafe {
			ChannelLayoutIter::new((*self.codec.as_ptr()).channel_layouts)
		}
	}
}

impl<'a> Deref for Audio<'a> {
	type Target = Codec<'a>;

	fn deref(&self) -> &Self::Target {
		self.codec
	}
}

pub struct RateIter<'a> {
	ptr: *const i32,

	_marker: PhantomData<&'a ()>,
}

impl<'a> RateIter<'a> {
	pub fn new(ptr: *const i32) -> Self {
		RateIter { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Iterator for RateIter<'a> {
	type Item = i32;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if !self.ptr.is_null() && (*self.ptr) != 0 {
				let rate = *self.ptr;
				self.ptr = self.ptr.offset(1);

				Some(rate)
			}
			else {
				None
			}
		}
	}
}

pub struct FormatIter<'a> {
	ptr: *const AVSampleFormat,

	_marker: PhantomData<&'a ()>,
}

impl<'a> FormatIter<'a> {
	pub fn new(ptr: *const AVSampleFormat) -> Self {
		FormatIter { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Iterator for FormatIter<'a> {
	type Item = format::Sample;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if !self.ptr.is_null() && (*self.ptr) != AVSampleFormat::AV_SAMPLE_FMT_NONE {
				let format = (*self.ptr).into();
				self.ptr   = self.ptr.offset(1);

				Some(format)
			}
			else {
				None
			}
		}
	}
}

pub struct ChannelLayoutIter<'a> {
	ptr: *const u64,

	_marker: PhantomData<&'a ()>,
}

impl<'a> ChannelLayoutIter<'a> {
	pub fn new(ptr: *const u64) -> Self {
		ChannelLayoutIter { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Iterator for ChannelLayoutIter<'a> {
	type Item = ChannelLayout;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if !self.ptr.is_null() && (*self.ptr) != 0 {
				let chl  = ChannelLayout::from_bits_truncate(*self.ptr);
				self.ptr = self.ptr.offset(1);

				Some(chl)
			}
			else {
				None
			}
		}
	}
}
