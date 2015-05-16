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

use std::ffi::CString;
use std::ptr;
use std::ops::Deref;

use ffi::*;
use super::Id;
use super::context::Opened;
use ::{Codec, Error};
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
