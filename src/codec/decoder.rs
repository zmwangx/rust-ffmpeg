use std::ffi::CString;
use std::ptr;

use ffi::*;
use ::codec::Id;
use ::{Codec, Packet, Subtitle, Error};
use ::frame;

pub trait Decode {
	fn video(&self, packet: &Packet, out: &mut frame::Video) -> Result<bool, Error>;
	fn audio(&self, packet: &Packet, out: &mut frame::Audio) -> Result<bool, Error>;
	fn subtitle(&self, packet: &Packet, out: &mut Subtitle) -> Result<bool, Error>;
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
