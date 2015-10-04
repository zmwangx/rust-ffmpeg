use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ffi::CString;

use ffi::*;
use ::{Error, StreamMut, Dictionary, format};
use super::common::Context;
use super::destructor;
use codec::traits;

pub struct Output {
	ptr: *mut AVFormatContext,
	ctx: Context,
}

unsafe impl Send for Output { }

impl Output {
	pub unsafe fn wrap(ptr: *mut AVFormatContext) -> Self {
		Output { ptr: ptr, ctx: Context::wrap(ptr, destructor::Mode::Output) }
	}

	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}
}

impl Output {
	pub fn format(&self) -> format::Output {
		unsafe {
			format::Output::wrap((*self.as_ptr()).oformat)
		}
	}

	pub fn write_header(&mut self) -> Result<(), Error> {
		unsafe {
			match avformat_write_header(self.as_mut_ptr(), ptr::null_mut()) {
				0 => Ok(()),
				e => Err(Error::from(e)),
			}
		}
	}

	pub fn write_header_with(&mut self, options: Dictionary) -> Result<(), Error> {
		unsafe {
			let mut opts = options.disown();
			let     res  = avformat_write_header(self.as_mut_ptr(), &mut opts);

			Dictionary::own(opts);

			match res {
				0 => Ok(()),
				e => Err(Error::from(e)),
			}
		}
	}

	pub fn write_trailer(&mut self) -> Result<(), Error> {
		unsafe {
			match av_write_trailer(self.as_mut_ptr()) {
				0 => Ok(()),
				e => Err(Error::from(e)),
			}
		}
	}

	pub fn add_stream<E: traits::Encoder>(&mut self, codec: E) -> Result<StreamMut, Error> {
		unsafe {
			let codec = try!(codec.encoder().ok_or(Error::EncoderNotFound));
			let ptr   = avformat_new_stream(self.as_mut_ptr(), codec.as_ptr());

			if ptr.is_null() {
				panic!("out of memory");
			}

			let index = (*self.ctx.as_ptr()).nb_streams - 1;

			Ok(StreamMut::wrap(&mut self.ctx, index as usize))
		}
	}

	pub fn set_metadata(&mut self, dictionary: Dictionary) {
		unsafe {
			(*self.as_mut_ptr()).metadata = dictionary.disown();
		}
	}
}

impl Deref for Output {
	type Target = Context;

	fn deref(&self) -> &Self::Target {
		&self.ctx
	}
}

impl DerefMut for Output {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.ctx
	}
}

pub fn dump(ctx: &Output, index: i32, url: Option<&str>) {
	let url = url.map(|u| CString::new(u).unwrap());

	unsafe {
		av_dump_format(ctx.as_ptr(), index,
			url.map(|u| u.as_ptr()).unwrap_or(ptr::null()), 1);
	}
}
