use std::ops::{Deref, DerefMut};

use libc::c_int;
use codec::Context;
use ::{Error, Rational, media};
use super::{video, audio, subtitle};

pub struct Encoder(pub Context);

impl Encoder {
	pub fn video(self) -> Result<video::Video, Error> {
		if self.medium() == media::Type::Video {
			Ok(video::Video(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}

	pub fn audio(self) -> Result<audio::Audio, Error> {
		if self.medium() == media::Type::Audio {
			Ok(audio::Audio(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}

	pub fn subtitle(self) -> Result<subtitle::Subtitle, Error> {
		if self.medium() == media::Type::Subtitle {
			Ok(subtitle::Subtitle(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}

	pub fn set_bit_rate(&mut self, value: usize) -> &mut Self {
		unsafe {
			(*self.as_mut_ptr()).bit_rate = value as c_int;
		}

		self
	}

	pub fn set_max_bit_rate(&mut self, value: usize) -> &mut Self {
		unsafe {
			(*self.as_mut_ptr()).rc_max_rate = value as c_int;
		}

		self
	}

	pub fn set_tolerance(&mut self, value: usize) -> &mut Self {
		unsafe {
			(*self.as_mut_ptr()).bit_rate_tolerance = value as c_int;
		}

		self
	}

	pub fn set_quality(&mut self, value: usize) -> &mut Self {
		unsafe {
			(*self.as_mut_ptr()).global_quality = value as c_int;
		}

		self
	}

	pub fn set_compression(&mut self, value: Option<usize>) -> &mut Self {
		unsafe {
			if let Some(value) = value {
				(*self.as_mut_ptr()).compression_level = value as c_int;
			}
			else {
				(*self.as_mut_ptr()).compression_level = -1;
			}
		}

		self
	}

	pub fn set_time_base<R: Into<Rational>>(&mut self, value: R) -> &mut Self {
		unsafe {
			(*self.as_mut_ptr()).time_base = value.into().into();
		}

		self
	}

	pub fn set_frame_rate<R: Into<Rational>>(&mut self, value: Option<R>) -> &mut Self {
		unsafe {
			if let Some(value) = value {
				(*self.as_mut_ptr()).framerate = value.into().into();
			}
			else {
				(*self.as_mut_ptr()).framerate.num = 0;
				(*self.as_mut_ptr()).framerate.den = 1;
			}
		}

		self
	}
}

impl Deref for Encoder {
	type Target = Context;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Encoder {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
