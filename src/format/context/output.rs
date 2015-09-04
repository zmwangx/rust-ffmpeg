use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ffi::CString;

use ffi::*;
use ::{Error, Codec, StreamMut, Dictionary};
use super::common::Context;

pub struct Output {
	ptr: *mut AVFormatContext,
	ctx: Context,
}

unsafe impl Send for Output { }

impl Output {
	pub unsafe fn wrap(ptr: *mut AVFormatContext) -> Self {
		Output { ptr: ptr, ctx: Context::wrap(ptr) }
	}

	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}
}

impl Output {
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
			let mut opts   = options.take();
			let     status = avformat_write_header(self.as_mut_ptr(), &mut opts);

			Dictionary::own(opts);

			match status {
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

	pub fn add_stream(&mut self, codec: &Codec) -> StreamMut {
		unsafe {
			let ptr = avformat_new_stream(self.as_mut_ptr(), codec.as_ptr());

			if ptr.is_null() {
				panic!("out of memory");
			}

			StreamMut::wrap(ptr)
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

impl Drop for Output {
	fn drop(&mut self) {
		unsafe {
			avformat_free_context(self.as_mut_ptr());
		}
	}
}

pub fn dump(ctx: &Output, index: i32, url: Option<&str>) {
	let url = url.map(|u| CString::new(u).unwrap());

	unsafe {
		av_dump_format(ctx.as_ptr(), index,
			url.map(|u| u.as_ptr()).unwrap_or(ptr::null()), 1);
	}
}
