use std::mem;
use std::marker::PhantomData;

use libc::{c_int, size_t};
use ffi::*;
use ::format;
use ::Error;

pub struct Picture<'a> {
	pub ptr: *mut AVPicture,

	pub format: format::Pixel,
	pub width:  usize,
	pub height: usize,

	_own:    bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Picture<'a> {
	pub fn size(format: format::Pixel, width: usize, height: usize) -> Result<usize, Error> {
		unsafe {
			match avpicture_get_size(format.into(), width as c_int, height as c_int) {
				v if v >= 0 => Ok(v as usize),
				e           => Err(Error::new(e))
			}
		}
	}

	pub fn new(format: format::Pixel, width: usize, height: usize) -> Result<Self, Error> {
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

				e => Err(Error::new(e))
			}
		}
	}

	pub fn wrap(ptr: *mut AVPicture, format: format::Pixel, width: usize, height: usize) -> Self {
		Picture {
			ptr: ptr,

			format: format,
			width:  width,
			height: height,
			
			_own:    false,
			_marker: PhantomData
		}
	}

	pub fn layout(&self, out: &mut [u8]) -> Result<usize, Error> {
		unsafe {
			match avpicture_layout(self.ptr, self.format.into(), self.width as c_int, self.height as c_int, out.as_mut_ptr(), out.len() as c_int) {
				s if s >= 0 => Ok(s as usize),
				e           => Err(Error::new(e))
			}
		}
	}

	pub fn layout_as(&self, format: format::Pixel, width: usize, height: usize, out: &mut [u8]) -> Result<usize, Error> {
		unsafe {
			match avpicture_layout(self.ptr, format.into(), width as c_int, height as c_int, out.as_mut_ptr(), out.len() as c_int) {
				s if s >= 0 => Ok(s as usize),
				e           => Err(Error::new(e))
			}
		}
	}


	pub fn crop(&mut self, source: &Picture, top: usize, left: usize) -> Result<(), Error> {
		if self.format != source.format {
			return Err(Error::new(AVERROR_BUG));
		}

		unsafe {
			match av_picture_crop(self.ptr, source.ptr, self.format.into(), top as c_int, left as c_int) {
				0 => Ok(()),
				e => Err(Error::new(e))
			}
		}
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
