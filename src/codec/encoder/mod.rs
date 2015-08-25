pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

pub mod subtitle;
pub use self::subtitle::Subtitle;

pub mod motion_estimation;
pub use self::motion_estimation::MotionEstimation;

pub mod prediction;
pub use self::prediction::Prediction;

pub mod comparison;
pub use self::comparison::Comparison;

pub mod decision;
pub use self::decision::Decision;

use std::ffi::CString;
use std::ops::{Deref, DerefMut};

use libc::c_int;
use ffi::*;
use super::{Id, Context};
use ::{Codec, Error, Rational};
use ::media;

pub struct Encoder(pub Context);

impl Encoder {
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

	pub fn set_rate(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).bit_rate = value as c_int;
		}
	}

	pub fn set_tolerance(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).bit_rate_tolerance = value as c_int;
		}
	}

	pub fn set_quality(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).global_quality = value as c_int;
		}
	}

	pub fn set_compression(&mut self, value: Option<usize>) {
		unsafe {
			if let Some(value) = value {
				(*self.as_mut_ptr()).compression_level = value as c_int;
			}
			else {
				(*self.as_mut_ptr()).compression_level = -1;
			}
		}
	}

	pub fn set_frame_rate<R: Into<Rational>>(&mut self, value: Option<R>) {
		unsafe {
			if let Some(value) = value {
				(*self.as_mut_ptr()).framerate = value.into().into();
			}
			else {
				(*self.as_mut_ptr()).framerate.num = 0;
				(*self.as_mut_ptr()).framerate.den = 1;
			}
		}
	}

	pub fn set_time_base<R: Into<Rational>>(&mut self, value: R) {
		unsafe {
			(*self.as_mut_ptr()).time_base = value.into().into();
		}
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

pub fn find(id: Id) -> Option<Codec<'static>> {
	unsafe {
		let ptr = avcodec_find_encoder(id.into());

		if ptr.is_null() {
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

		if ptr.is_null() {
			None
		}
		else {
			Some(Codec::wrap(ptr))
		}
	}
}
