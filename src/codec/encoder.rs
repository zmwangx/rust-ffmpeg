use std::ffi::CString;
use std::ptr;
use std::ops::Deref;

use libc::c_int;
use ffi::*;
use super::Id;
use super::context::Opened;
use ::{Codec, Packet, Error};
use ::frame;
use ::media;

pub struct Encoder(pub Opened);

impl Encoder {
	pub fn video(self) -> Result<Video, Error> {
		if self.medium() == media::Type::Video {
			Ok(Video(self))
		}
		else {
			Err(Error::from(AVERROR_INVALIDDATA))
		}
	}

	pub fn audio(self) -> Result<Audio, Error> {
		if self.medium() == media::Type::Audio {
			Ok(Audio(self))
		}
		else {
			Err(Error::from(AVERROR_INVALIDDATA))
		}
	}

	pub fn subtitle(self) -> Result<Subtitle, Error> {
		if self.medium() == media::Type::Subtitle {
			Ok(Subtitle(self))
		}
		else {
			Err(Error::from(AVERROR_INVALIDDATA))
		}
	}
}

impl Deref for Encoder {
	type Target = Opened;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct Video(pub Encoder);

impl Video {
	pub fn encode(&self, frame: &frame::Video, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_video2(self.ptr, &mut out.val, frame.ptr, &mut got) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Video {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct Audio(pub Encoder);

impl Audio {
	pub fn encode(&self, frame: &frame::Audio, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_audio2(self.ptr, &mut out.val, frame.ptr, &mut got) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Audio {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct Subtitle(pub Encoder);

impl Subtitle {
	pub fn encode(&self, subtitle: &::Subtitle, out: &mut [u8]) -> Result<bool, Error> {
		unsafe {
			match avcodec_encode_subtitle(self.ptr, out.as_mut_ptr(), out.len() as c_int, &subtitle.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(true)
			}
		}
	}
}

impl Deref for Subtitle {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub fn find(id: Id) -> Option<Codec<'static>> {
	unsafe {
		let ptr = avcodec_find_encoder(id.into());

		if ptr == ptr::null_mut() {
			None
		}
		else {
			Some(Codec::wrap(ptr))
		}
	}
}

pub fn find_by_name(name: &str) -> Option<Codec<'static>> {
	unsafe {
		let ptr = avcodec_find_encoder_by_name(CString::new(name).unwrap().as_ptr());

		if ptr == ptr::null_mut() {
			None
		}
		else {
			Some(Codec::wrap(ptr))
		}
	}
}
