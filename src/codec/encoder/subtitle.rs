use std::ops::{Deref, DerefMut};

use libc::c_int;
use ffi::*;

use super::Encoder;
use ::Error;

pub struct Subtitle(pub Encoder);

impl Subtitle {
	pub fn encode(&mut self, subtitle: &::Subtitle, out: &mut [u8]) -> Result<bool, Error> {
		unsafe {
			match avcodec_encode_subtitle(self.as_mut_ptr(), out.as_mut_ptr(), out.len() as c_int, subtitle.as_ptr()) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(true)
			}
		}
	}
}

impl Deref for Subtitle {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Subtitle {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
