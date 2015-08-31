use std::ops::{Deref, DerefMut};
use std::ptr;

use libc::c_int;
use ffi::*;

use super::Encoder as Super;
use ::{Packet, Error, Dictionary, Codec, ChannelLayout};
use ::frame;
use ::util::format;

pub struct Audio(pub Super);

impl Audio {
	pub fn open(mut self) -> Result<Encoder, Error> {
		unsafe {
			match avcodec_open2(self.as_mut_ptr(), ptr::null(), ptr::null_mut()) {
				0 => Ok(Encoder(self)),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn open_as(mut self, codec: &Codec) -> Result<Encoder, Error> {
		unsafe {
			if codec.is_encoder() {
				match avcodec_open2(self.as_mut_ptr(), codec.as_ptr(), ptr::null_mut()) {
					0 => Ok(Encoder(self)),
					e => Err(Error::from(e))
				}
			}
			else {
				Err(Error::InvalidData)
			}
		}
	}

	pub fn open_as_with(mut self, codec: &Codec, options: Dictionary) -> Result<Encoder, Error> {
		unsafe {
			if codec.is_encoder() {
				match avcodec_open2(self.as_mut_ptr(), codec.as_ptr(), &mut options.take()) {
					0 => Ok(Encoder(self)),
					e => Err(Error::from(e))
				}
			}
			else {
				Err(Error::InvalidData)
			}
		}
	}

	pub fn set_rate(&mut self, rate: i32) {
		unsafe {
			(*self.as_mut_ptr()).sample_rate = rate;
		}
	}

	pub fn set_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.as_mut_ptr()).sample_fmt = value.into();
		}
	}

	pub fn set_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			(*self.as_mut_ptr()).channel_layout = value.bits();
		}
	}

	pub fn set_channels(&mut self, value: i32) {
		unsafe {
			(*self.as_mut_ptr()).channels = value;
		}
	}
}

impl Deref for Audio {
	type Target = Super;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Audio {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}

pub struct Encoder(pub Audio);

impl Encoder {
	pub fn encode(&mut self, frame: &frame::Audio, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_audio2(self.0.as_mut_ptr(), out.as_mut_ptr(), frame.as_ptr(), &mut got) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Encoder {
	type Target = Audio;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}
