use std::ptr;

use libc::c_int;
use ffi::*;
use ::media;
use ::Codec;
use super::{Id, Debug, Compliance, threading};
use super::decoder::Decoder;
use super::encoder::Encoder;

pub struct Context {
	ptr: *mut AVCodecContext,

	_own: bool,
}

unsafe impl Send for Context { }

impl Context {
	pub unsafe fn wrap(ptr: *mut AVCodecContext) -> Self {
		Context { ptr: ptr, _own: false }
	}

	pub unsafe fn as_ptr(&self) -> *const AVCodecContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
		self.ptr
	}
}

impl Context {
	pub fn new() -> Self {
		unsafe {
			Context { ptr: avcodec_alloc_context3(ptr::null()), _own: true }
		}
	}

	pub fn decoder(self) -> Decoder {
		Decoder(self)
	}

	pub fn encoder(self) -> Encoder {
		Encoder(self)
	}

	pub fn codec(&self) -> Option<Codec> {
		unsafe {
			if (*self.as_ptr()).codec.is_null() {
				None
			}
			else {
				Some(Codec::wrap((*self.as_ptr()).codec as *mut _))
			}
		}
	}

	pub fn medium(&self) -> media::Type {
		unsafe {
			media::Type::from((*self.as_ptr()).codec_type)
		}
	}

	pub fn id(&self) -> Id {
		unsafe {
			Id::from((*self.as_ptr()).codec_id)
		}
	}

	pub fn compliance(&mut self, value: Compliance) {
		unsafe {
			(*self.as_mut_ptr()).strict_std_compliance = value.into();
		}
	}

	pub fn debug(&mut self, value: Debug) {
		unsafe {
			(*self.as_mut_ptr()).debug = value.bits();
		}
	}

	pub fn set_threading(&mut self, config: threading::Config) {
		unsafe {
			(*self.as_mut_ptr()).thread_type           = config.kind.into();
			(*self.as_mut_ptr()).thread_count          = config.count as c_int;
			(*self.as_mut_ptr()).thread_safe_callbacks = if config.safe { 1 } else { 0 };
		}
	}

	pub fn threading(&self) -> threading::Config {
		unsafe {
			threading::Config {
				kind:  threading::Type::from((*self.as_ptr()).active_thread_type),
				count: (*self.as_ptr()).thread_count as usize,
				safe:  (*self.as_ptr()).thread_safe_callbacks != 0,
			}
		}
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		if self._own {
			unsafe {
				avcodec_free_context(&mut self.as_mut_ptr());
			}
		}
	}
}

impl Clone for Context {
	fn clone(&self) -> Self {
		let mut ctx = Context::new();
		ctx.clone_from(self);

		ctx
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			avcodec_copy_context(self.as_mut_ptr(), source.as_ptr());
		}
	}
}
