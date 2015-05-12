use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;
use libc::c_int;

use ffi::*;
use ::media;
use ::{Error, Codec, Dictionary, Packet, Subtitle};
use super::{Id, Encode, Decode};
use ::frame;

pub struct Context<'a> {
	pub ptr: *mut AVCodecContext,

	_own:    bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Context<'a> {
	pub fn new() -> Self {
		unsafe {
			Context { ptr: avcodec_alloc_context3(ptr::null()), _own: true, _marker: PhantomData }
		}
	}

	pub fn wrap(ptr: *mut AVCodecContext) -> Self {
		Context { ptr: ptr, _own: false, _marker: PhantomData }
	}

	pub fn open(self, codec: &Codec) -> Result<Opened<'a>, Error> {
		unsafe {
			match avcodec_open2(self.ptr, codec.ptr, ptr::null_mut()) {
				0 => Ok(Opened(self)),
				e => Err(Error::new(e))
			}
		}
	}

	pub fn open_with(self, codec: &Codec, mut options: Dictionary) -> Result<Opened<'a>, Error> {
		unsafe {
			match avcodec_open2(self.ptr, codec.ptr, &mut options.ptr) {
				0 => Ok(Opened(self)),
				e => Err(Error::new(e))
			}
		}
	}

	pub fn kind(&self) -> media::Type {
		unsafe {
			media::Type::from((*self.ptr).codec_type)
		}
	}

	pub fn id(&self) -> Id {
		unsafe {
			Id::from((*self.ptr).codec_id)
		}
	}
}

impl<'a> Drop for Context<'a> {
	fn drop(&mut self) {
		if self._own {
			unsafe {
				avcodec_free_context(&mut self.ptr);
			}
		}
	}
}

impl<'a> Clone for Context<'a> {
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

impl<'a> Decode for Context<'a> {
	fn video(&self, packet: &Packet, out: &mut frame::Video) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_video2(self.ptr, out.ptr, &mut got, &packet.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}

	fn audio(&self, packet: &Packet, out: &mut frame::Audio) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_audio4(self.ptr, out.ptr, &mut got, &packet.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}

	fn subtitle(&self, packet: &Packet, out: &mut Subtitle) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_subtitle2(self.ptr, &mut out.val, &mut got, &packet.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl<'a> Encode for Context<'a> {
	fn video(&self, frame: &frame::Video, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_video2(self.ptr, &mut out.val, frame.ptr, &mut got) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}

	fn audio(&self, frame: &frame::Audio, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_audio2(self.ptr, &mut out.val, frame.ptr, &mut got) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}

	fn subtitle(&self, subtitle: &Subtitle, out: &mut [u8]) -> Result<bool, Error> {
		unsafe {
			match avcodec_encode_subtitle(self.ptr, out.as_mut_ptr(), out.len() as c_int, &subtitle.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(true)
			}
		}
	}
}

pub struct Opened<'a>(Context<'a>);

impl<'a> Decode for Opened<'a> {
	fn video(&self, packet: &Packet, out: &mut frame::Video) -> Result<bool, Error> {
		Decode::video(&self.0, packet, out)
	}

	fn audio(&self, packet: &Packet, out: &mut frame::Audio) -> Result<bool, Error> {
		Decode::audio(&self.0, packet, out)
	}

	fn subtitle(&self, packet: &Packet, out: &mut Subtitle) -> Result<bool, Error> {
		Decode::subtitle(&self.0, packet, out)
	}
}

impl<'a> Encode for Opened<'a> {
	fn video(&self, frame: &frame::Video, out: &mut Packet) -> Result<bool, Error> {
		Encode::video(&self.0, frame, out)
	}

	fn audio(&self, frame: &frame::Audio, out: &mut Packet) -> Result<bool, Error> {
		Encode::audio(&self.0, frame, out)
	}

	fn subtitle(&self, subtitle: &Subtitle, out: &mut [u8]) -> Result<bool, Error> {
		Encode::subtitle(&self.0, subtitle, out)
	}
}

impl<'a> Drop for Opened<'a> {
	fn drop(&mut self) {
		unsafe {
			avcodec_close(self.0.ptr);
		}
	}
}

impl<'a> Deref for Opened<'a> {
	type Target = Context<'a>;

	fn deref(&self) -> &Context<'a> {
		&self.0
	}
}
