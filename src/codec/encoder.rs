use std::ffi::CString;
use std::ptr;

use ffi::*;
use ::codec::Id;
use ::{Codec, Packet, Subtitle, Error};
use ::frame;

pub trait Encode {
	fn video(&self, frame: &frame::Video, out: &mut Packet) -> Result<bool, Error>;
	fn audio(&self, frame: &frame::Audio, out: &mut Packet) -> Result<bool, Error>;
	fn subtitle(&self, subtitle: &Subtitle, out: &mut [u8]) -> Result<bool, Error>;
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
