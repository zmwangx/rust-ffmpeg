use std::ops::{Deref, DerefMut};

use ffi::*;
use super::{Video, Audio, Subtitle, Decoder};
use ::codec::Profile;
use ::{Error, Rational};
use ::media;

pub struct Opened(pub Decoder);

impl Opened {
	pub fn video(self) -> Result<Video, Error> {
		if self.medium() == media::Type::Video {
			Ok(Video(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}

	pub fn audio(self) -> Result<Audio, Error> {
		if self.medium() == media::Type::Audio {
			Ok(Audio(self))
		}
		else {
			Err(Error::InvalidData)
		}
	}

	pub fn subtitle(self) -> Result<Subtitle, Error> {
		if self.medium() == media::Type::Subtitle {
			Ok(Subtitle(self))
		}
		else {
			Err(Error::InvalidData)
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

	pub fn profile(&self) -> Profile {
		unsafe {
			Profile::from((self.id(), (*self.as_ptr()).profile))
		}
	}

	pub fn frame_rate(&self) -> Option<Rational> {
		unsafe {
			let value = (*self.as_ptr()).framerate;

			if value == (AVRational { num: 0, den: 1 }) {
				None
			}
			else {
				Some(Rational::from(value))
			}
		}
	}

	pub fn flush(&mut self) {
		unsafe {
			avcodec_flush_buffers(self.as_mut_ptr());
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
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Opened {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
