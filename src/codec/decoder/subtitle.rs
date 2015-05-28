use std::ops::{Deref, DerefMut};

use libc::c_int;
use ffi::*;

use super::Decoder;
use ::{Packet, Error};

pub struct Subtitle(pub Decoder);

impl Subtitle {
	pub fn decode(&mut self, packet: &Packet, out: &mut ::Subtitle) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_subtitle2(self.ptr, &mut out.val, &mut got, &packet.val) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Subtitle {
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Subtitle {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
