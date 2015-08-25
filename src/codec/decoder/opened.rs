use std::ops::{Deref, DerefMut};
use std::slice::from_raw_parts;

use ffi::*;
use super::{Conceal, Check, Video, Audio, Subtitle, Decoder};
use ::codec::Profile;
use ::{Error, Discard, Rational};
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

	pub fn conceal(&mut self, value: Conceal) {
		unsafe {
			(*self.as_mut_ptr()).error_concealment = value.bits();
		}
	}

	pub fn check(&mut self, value: Check) {
		unsafe {
			(*self.as_mut_ptr()).err_recognition = value.bits();
		}
	}

	pub fn profile(&self) -> Profile {
		unsafe {
			Profile::from((self.id(), (*self.as_ptr()).profile))
		}
	}

	pub fn skip_loop_filter(&mut self, value: Discard) {
		unsafe {
			(*self.as_mut_ptr()).skip_loop_filter = value.into();
		}
	}

	pub fn skip_idct(&mut self, value: Discard) {
		unsafe {
			(*self.as_mut_ptr()).skip_idct = value.into();
		}
	}

	pub fn skip_frame(&mut self, value: Discard) {
		unsafe {
			(*self.as_mut_ptr()).skip_frame = value.into();
		}
	}

	pub fn subtitle_header(&self) -> &[u8] {
		unsafe {
			from_raw_parts((*self.as_ptr()).subtitle_header, (*self.as_ptr()).subtitle_header_size as usize)
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

	pub fn time_base(&self) -> Rational {
		unsafe {
			Rational::from((*self.as_ptr()).time_base)
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
