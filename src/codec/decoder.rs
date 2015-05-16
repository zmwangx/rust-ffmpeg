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

pub struct Decoder(pub Opened);

impl Decoder {
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

impl Deref for Decoder {
	type Target = Opened;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct Video(pub Decoder);

impl Video {
	pub fn decode(&self, packet: &Packet, out: &mut frame::Video) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_video2(self.ptr, out.ptr, &mut got, &packet.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Video {
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct Audio(pub Decoder);

impl Audio {
	pub fn decode(&self, packet: &Packet, out: &mut frame::Audio) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_audio4(self.ptr, out.ptr, &mut got, &packet.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Audio {
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct Subtitle(pub Decoder);

impl Subtitle {
	pub fn decode(&self, packet: &Packet, out: &mut ::Subtitle) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_subtitle2(self.ptr, &mut out.val, &mut got, &packet.val) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Subtitle {
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub fn find(id: Id) -> Option<Codec<'static>> {
	unsafe {
		let ptr = avcodec_find_decoder(id.into());

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
		let ptr = avcodec_find_decoder_by_name(CString::new(name).unwrap().as_ptr());

		if ptr == ptr::null_mut() {
			None
		}
		else {
			Some(Codec::wrap(ptr))
		}
	}
}
