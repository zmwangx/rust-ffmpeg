pub mod decoder;
pub use self::decoder::Decoder;

pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

pub mod subtitle;
pub use self::subtitle::Subtitle;

pub mod slice;

pub mod conceal;
pub use self::conceal::Conceal;

pub mod check;
pub use self::check::Check;

pub mod opened;
pub use self::opened::Opened;

use std::ffi::CString;

use ffi::*;
use codec::Context;
use ::Codec;
use codec::Id;

pub fn new() -> Decoder {
	Context::new().decoder()
}

pub fn find(id: Id) -> Option<Codec> {
	unsafe {
		let ptr = avcodec_find_decoder(id.into());

		if ptr.is_null() {
			None
		}
		else {
			Some(Codec::wrap(ptr))
		}
	}
}

pub fn find_by_name(name: &str) -> Option<Codec> {
	unsafe {
		let name = CString::new(name).unwrap();
		let ptr  = avcodec_find_decoder_by_name(name.as_ptr());

		if ptr.is_null() {
			None
		}
		else {
			Some(Codec::wrap(ptr))
		}
	}
}
