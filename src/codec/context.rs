use std::ops::Deref;
use std::ptr;

use libc::c_int;
use ffi::*;
use ::media;
use ::{Error, Codec, Dictionary};
use super::{Id, Debug, Compliance, threading};
use super::decoder::Decoder;
use super::encoder::Encoder;

pub struct Context {
	pub ptr: *mut AVCodecContext,

	_own: bool,
}

impl Context {
	pub fn new() -> Self {
		unsafe {
			Context { ptr: avcodec_alloc_context3(ptr::null()), _own: true }
		}
	}

	pub fn wrap(ptr: *mut AVCodecContext) -> Self {
		Context { ptr: ptr, _own: false }
	}

	pub fn open(self, codec: &Codec) -> Result<Opened, Error> {
		unsafe {
			match avcodec_open2(self.ptr, codec.ptr, ptr::null_mut()) {
				0 => Ok(Opened(self)),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn open_with(self, codec: &Codec, mut options: Dictionary) -> Result<Opened, Error> {
		unsafe {
			match avcodec_open2(self.ptr, codec.ptr, &mut options.ptr) {
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
			if (*self.ptr).codec == ptr::null() {
				None
			}
			else {
				Some(Codec::wrap((*self.ptr).codec as *mut _))
			}
		}
	}

	pub fn medium(&self) -> media::Type {
		unsafe {
			media::Type::from((*self.ptr).codec_type)
		}
	}

	pub fn id(&self) -> Id {
		unsafe {
			Id::from((*self.ptr).codec_id)
		}
	}

	pub fn bit_rate(&self) -> usize {
		unsafe {
			(*self.ptr).bit_rate as usize
		}
	}

	pub fn delay(&self) -> usize {
		unsafe {
			(*self.ptr).delay as usize
		}
	}

	pub fn compliance(&mut self, value: Compliance) {
		unsafe {
			(*self.ptr).strict_std_compliance = value.into();
		}
	}

	pub fn debug(&mut self, value: Debug) {
		unsafe {
			(*self.ptr).debug = value.bits();
		}
	}

	pub fn set_threading(&mut self, config: threading::Config) {
		unsafe {
			(*self.ptr).thread_type           = config.kind.into();
			(*self.ptr).thread_count          = config.count as c_int;
			(*self.ptr).thread_safe_callbacks = if config.safe { 1 } else { 0 };
		}
	}

	pub fn threading(&self) -> threading::Config {
		unsafe {
			threading::Config {
				kind:  threading::Type::from((*self.ptr).active_thread_type),
				count: (*self.ptr).thread_count as usize,
				safe:  (*self.ptr).thread_safe_callbacks != 0,
			}
		}
	}
}

unsafe impl Send for Context { }

impl Drop for Context {
	fn drop(&mut self) {
		if self._own {
			unsafe {
				avcodec_free_context(&mut self.ptr);
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
			avcodec_copy_context(self.ptr, source.ptr);
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
			avcodec_close(self.0.ptr);
		}
	}
}

impl Deref for Opened {
	type Target = Context;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}
