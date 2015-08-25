use std::marker::PhantomData;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;
use super::{Id, Video, Audio, Capabilities, Profile};
use ::{Error, media};

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

	pub fn max_lowres(&self) -> i32 {
		unsafe {
			av_codec_get_max_lowres(self.as_ptr())
		}
	}

	pub fn capabilities(&self) -> Capabilities {
		unsafe {
			Capabilities::from_bits_truncate((*self.as_ptr()).capabilities as u32)
		}
	}

	pub fn profiles(&self) -> Option<ProfileIter> {
		unsafe {
			if (*self.as_ptr()).profiles.is_null() {
				None
			}
			else {
				Some(ProfileIter::new(self.id(), (*self.as_ptr()).profiles))
			}
		}
	}
}

pub struct ProfileIter<'a> {
	id:  Id,
	ptr: *const AVProfile,

	_marker: PhantomData<&'a ()>,
}

impl<'a> ProfileIter<'a> {
	pub fn new(id: Id, ptr: *const AVProfile) -> Self {
		ProfileIter { id: id, ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Iterator for ProfileIter<'a> {
	type Item = Profile;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if (*self.ptr).profile == FF_PROFILE_UNKNOWN {
				return None;
			}

			let profile = Profile::from((self.id, (*self.ptr).profile));
			self.ptr    = self.ptr.offset(1);

			Some(profile)
		}
	}
}
