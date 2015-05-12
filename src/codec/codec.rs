use std::marker::PhantomData;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;
use super::{Id, Context};
use ::media;
use ::Error;
use ::codec::context::Opened;

pub struct Codec<'a> {
	pub ptr: *mut AVCodec,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Codec<'a> {
	pub fn wrap(ptr: *mut AVCodec) -> Self {
		Codec { ptr: ptr, _marker: PhantomData }
	}

	pub fn open(&self) -> Result<Opened<'a>, Error> {
		Context::new().open(self)
	}

	pub fn is_encoder(&self) -> bool {
		unsafe {
			av_codec_is_encoder(self.ptr) != 0
		}
	}

	pub fn is_decoder(&self) -> bool {
		unsafe {
			av_codec_is_decoder(self.ptr) != 0
		}
	}

	pub fn name(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).name).to_bytes())
		}
	}

	pub fn description(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).long_name).to_bytes())
		}
	}

	pub fn kind(&self) -> media::Type {
		unsafe {
			media::Type::from((*self.ptr).kind)
		}
	}

	pub fn id(&self) -> Id {
		unsafe {
			Id::from((*self.ptr).id)
		}
	}
}
