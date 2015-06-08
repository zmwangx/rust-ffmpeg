use std::ops::{Deref, DerefMut};
use std::ptr;

use libc::c_int;
use ffi::*;
use ::media;
use ::{Error, Codec, Dictionary};
use super::{Id, Debug, Compliance, threading};
use super::decoder::Decoder;
use super::encoder::Encoder;

pub struct Context {
	ptr: *mut AVCodecContext,

	_own: bool,
}

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

	pub fn open(mut self, codec: &Codec) -> Result<Opened, Error> {
		unsafe {
			match avcodec_open2(self.as_mut_ptr(), codec.as_ptr(), ptr::null_mut()) {
				0 => Ok(Opened(self)),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn open_with(mut self, codec: &Codec, options: Dictionary) -> Result<Opened, Error> {
		unsafe {
			match avcodec_open2(self.as_mut_ptr(), codec.as_ptr(), &mut options.take()) {
				0 => Ok(Opened(self)),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn decoder(&self) -> Result<Decoder, Error> {
		if let Some(ref codec) = super::decoder::find(self.id()) {
			self.clone().open(codec).and_then(|c| c.decoder())
		}
		else {
			Err(Error::DecoderNotFound)
		}
	}

	pub fn encoder(&self) -> Result<Encoder, Error> {
		if let Some(ref codec) = super::encoder::find(self.id()) {
			self.clone().open(codec).and_then(|c| c.encoder())
		}
		else {
			Err(Error::EncoderNotFound)
		}
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

	pub fn bit_rate(&self) -> usize {
		unsafe {
			(*self.as_ptr()).bit_rate as usize
		}
	}

	pub fn delay(&self) -> usize {
		unsafe {
			(*self.as_ptr()).delay as usize
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

unsafe impl Send for Context { }

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

pub struct Opened(pub Context);

impl Opened {
	pub fn decoder(self) -> Result<Decoder, Error> {
		let mut valid = false;

		if let Some(codec) = self.codec() {
			valid = codec.is_decoder();
		}

		if valid {
			Ok(Decoder(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}

	pub fn encoder(self) -> Result<Encoder, Error> {
		let mut valid = false;

		if let Some(codec) = self.codec() {
			valid = codec.is_encoder();
		}

		if valid {
			Ok(Encoder(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}
}

impl Drop for Opened {
	fn drop(&mut self) {
		unsafe {
			avcodec_close(self.as_mut_ptr());
		}
	}
}

impl Deref for Opened {
	type Target = Context;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Opened {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
