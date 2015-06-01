use std::mem;
use std::slice;
use std::marker::{Reflect, PhantomData};

use libc::{c_int, size_t};
use ffi::*;
use ::util::format::Pixel;
use ::Error;

pub struct Picture<'a> {
	pub ptr: *mut AVPicture,

	format: Pixel,
	width:  u32,
	height: u32,

	_own:    bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Picture<'a> {
	pub fn size(format: Pixel, width: u32, height: u32) -> Result<usize, Error> {
		unsafe {
			match avpicture_get_size(format.into(), width as c_int, height as c_int) {
				v if v >= 0 => Ok(v as usize),
				e           => Err(Error::from(e))
			}
		}
	}

	pub fn new(format: Pixel, width: u32, height: u32) -> Result<Self, Error> {
		unsafe {
			let ptr = av_malloc(mem::size_of::<AVPicture>() as size_t) as *mut AVPicture;

			match avpicture_alloc(ptr, format.into(), width as c_int, height as c_int) {
				0 => Ok(Picture {
					ptr: ptr,

					format: format,
					width:  width,
					height: height,

					_own:    true,
					_marker: PhantomData
				}),

				e => Err(Error::from(e))
			}
		}
	}

	pub fn wrap(ptr: *mut AVPicture, format: Pixel, width: u32, height: u32) -> Self {
		Picture {
			ptr: ptr,

			format: format,
			width:  width,
			height: height,

			_own:    false,
			_marker: PhantomData
		}
	}

	pub fn format(&self) -> Pixel {
		self.format
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn layout(&self, out: &mut [u8]) -> Result<usize, Error> {
		unsafe {
			match avpicture_layout(self.ptr, self.format.into(), self.width as c_int, self.height as c_int, out.as_mut_ptr(), out.len() as c_int) {
				s if s >= 0 => Ok(s as usize),
				e           => Err(Error::from(e))
			}
		}
	}

	pub fn layout_as(&self, format: Pixel, width: u32, height: u32, out: &mut [u8]) -> Result<usize, Error> {
		unsafe {
			match avpicture_layout(self.ptr, format.into(), width as c_int, height as c_int, out.as_mut_ptr(), out.len() as c_int) {
				s if s >= 0 => Ok(s as usize),
				e           => Err(Error::from(e))
			}
		}
	}

	pub fn crop(&mut self, source: &Picture, top: u32, left: u32) -> Result<(), Error> {
		if self.format != source.format {
			return Err(Error::Bug);
		}

		unsafe {
			match av_picture_crop(self.ptr, source.ptr, self.format.into(), top as c_int, left as c_int) {
				0 => Ok(()),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn data<T: Reflect + 'static>(&self) -> Result<Vec<&[T]>, Error> {
		if !valid::<T>(self.format) {
			return Err(Error::InvalidData);
		}

		let mut result = Vec::new();

		unsafe {
			for (i, length) in (*self.ptr).linesize.iter().take_while(|l| **l > 0).enumerate() {
				result.push(slice::from_raw_parts(
					mem::transmute((*self.ptr).data[i]),
					((*length as usize) * (self.height as usize)) / mem::size_of::<T>()));
			}
		}

		Ok(result)
	}

	pub fn data_mut<T: Reflect + 'static>(&mut self) -> Result<Vec<&mut [u8]>, Error> {
		if !valid::<T>(self.format) {
			return Err(Error::InvalidData);
		}

		let mut result = Vec::new();

		unsafe {
			for (i, length) in (*self.ptr).linesize.iter().take_while(|l| **l > 0).enumerate() {
				result.push(slice::from_raw_parts_mut(
					mem::transmute((*self.ptr).data[i]),
					((*length as usize) * (self.height as usize)) / mem::size_of::<T>()));
			}
		}

		Ok(result)
	}
}

impl<'a> Clone for Picture<'a> {
	fn clone(&self) -> Self {
		let mut pic = Picture::new(self.format, self.width, self.height).unwrap();
		pic.clone_from(self);

		pic
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_picture_copy(self.ptr, source.ptr, source.format.into(), source.width as c_int, source.height as c_int);
		}
	}
}

impl<'a> Drop for Picture<'a> {
	fn drop(&mut self) {
		if self._own {
			unsafe {
				av_free(mem::transmute(self.ptr));
			}
		}
	}
}

pub fn valid<T: Reflect + 'static>(format: Pixel) -> bool {
	if mem::size_of::<T>() == 1 {
		return true;
	}

	match format {
		Pixel::None =>
			false,

		Pixel::RGB24 | Pixel::BGR24 =>
			mem::size_of::<T>() == 3,

		Pixel::ARGB | Pixel::RGBA | Pixel::ABGR | Pixel::BGRA =>
			mem::size_of::<T>() == 4 * 4,

		Pixel::ZRGB | Pixel::RGBZ | Pixel::ZBGR | Pixel::BGRZ =>
			mem::size_of::<T>() == 4 * 4,

		_ =>
			false
	}
}
