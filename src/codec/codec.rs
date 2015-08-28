use std::marker::PhantomData;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;
use super::{Id, Context, Video, Audio};
use ::{Error, media};
use ::codec::context::Opened;

pub struct Codec<'a> {
	ptr: *mut AVCodec,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Codec<'a> {
	pub unsafe fn wrap(ptr: *mut AVCodec) -> Self {
		Codec { ptr: ptr, _marker: PhantomData }
	}

	pub unsafe fn as_ptr(&self) -> *const AVCodec {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodec {
		self.ptr
	}
}

impl<'a> Codec<'a> {
	pub fn open(&self) -> Result<Opened, Error> {
		Context::new().open(self)
	}

	pub fn is_encoder(&self) -> bool {
		unsafe {
			av_codec_is_encoder(self.as_ptr()) != 0
		}
	}

	pub fn is_decoder(&self) -> bool {
		unsafe {
			av_codec_is_decoder(self.as_ptr()) != 0
		}
	}

	pub fn name(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).name).to_bytes())
		}
	}

	pub fn description(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).long_name).to_bytes())
		}
	}

	pub fn medium(&self) -> media::Type {
		unsafe {
			media::Type::from((*self.as_ptr()).kind)
		}
	}

	pub fn id(&self) -> Id {
		unsafe {
			Id::from((*self.as_ptr()).id)
		}
	}

	pub fn video(&self) -> Result<Video, Error> {
		unsafe {
			if self.medium() == media::Type::Video {
				Ok(Video::new(self))
			}
			else {
				Err(Error::InvalidData)
			}
		}
	}

	pub fn audio(&self) -> Result<Audio, Error> {
		unsafe {
			if self.medium() == media::Type::Audio {
				Ok(Audio::new(self))
			}
			else {
				Err(Error::InvalidData)
			}
		}
	}

	// capabilities

	pub fn max_lowres(&self) -> i32 {
		unsafe {
			av_codec_get_max_lowres(self.as_ptr())
		}
	}

	// profiles
}
