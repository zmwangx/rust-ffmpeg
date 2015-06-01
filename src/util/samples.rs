use std::slice;
use std::mem;
use std::any::TypeId;
use std::marker::{Reflect, PhantomData};

use ffi::*;
use ::util::format::Sample;
use ::{Error, ChannelLayout};

pub struct Samples<'a> {
	pub ptr: *mut AVPicture,

	format:   Sample,
	rate:     u32,
	number:   usize,
	channels: u16,
	layout:   ChannelLayout,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Samples<'a> {
	pub fn wrap(ptr: *mut AVPicture, format: Sample, rate: u32, number: usize, channels: u16, layout: ChannelLayout) -> Self {
		Samples {
			ptr: ptr,

			format:   format,
			rate:     rate,
			number:   number,
			channels: channels,
			layout:   layout,

			_marker: PhantomData,
		}
	}

	pub fn format(&self) -> Sample {
		self.format
	}

	pub fn rate(&self) -> u32 {
		self.rate
	}

	pub fn number(&self) -> usize {
		self.number
	}

	pub fn channels(&self) -> u16 {
		self.channels
	}

	pub fn channel_layout(&self) -> ChannelLayout {
		self.layout
	}

	pub fn is_planar(&self) -> bool {
		self.format.is_planar()
	}

	pub fn is_packed(&self) -> bool {
		self.format.is_planar()
	}

	pub fn data<T: Reflect + 'static>(&self) -> Result<Vec<&[T]>, Error> {
		if !valid::<T>(self.format) {
			return Err(Error::InvalidData)
		}

		let mut result = Vec::new();

		unsafe {
			let size = (*self.ptr).linesize[0];

			if self.is_planar() {
				for i in 0 .. self.channels {
					result.push(slice::from_raw_parts(
						mem::transmute((*self.ptr).data[i as usize]),
						size as usize / mem::size_of::<T>()));
				}
			}
			else {
				result.push(slice::from_raw_parts(
					mem::transmute((*self.ptr).data[0]),
					size as usize / mem::size_of::<T>()));
			}
		}

		Ok(result)
	}

	pub fn data_mut<T: Reflect + 'static>(&mut self) -> Result<Vec<&mut [T]>, Error> {
		if !valid::<T>(self.format) {
			return Err(Error::InvalidData)
		}

		let mut result = Vec::new();

		unsafe {
			let size = (*self.ptr).linesize[0];

			if self.is_planar() {
				for i in 0 .. self.channels {
					result.push(slice::from_raw_parts_mut(
						mem::transmute((*self.ptr).data[i as usize]),
						size as usize / mem::size_of::<T>()));
				}
			}
			else {
				result.push(slice::from_raw_parts_mut(
					mem::transmute((*self.ptr).data[0]),
					size as usize / mem::size_of::<T>()));
			}
		}

		Ok(result)
	}
}

pub fn valid<T: Reflect + 'static>(format: Sample) -> bool {
	match format {
		Sample::None =>
			false,

		Sample::U8(..) if TypeId::of::<T>() != TypeId::of::<u8>() =>
			false,

		Sample::I16(..) if TypeId::of::<T>() != TypeId::of::<i16>() =>
			false,

		Sample::I32(..) if TypeId::of::<T>() != TypeId::of::<i32>() =>
			false,

		Sample::F32(..) if TypeId::of::<T>() != TypeId::of::<f32>() =>
			false,

		Sample::F64(..) if TypeId::of::<T>() != TypeId::of::<f64>() =>
			false,

		_ =>
			true
	}
}
