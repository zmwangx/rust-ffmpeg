use std::ops::Deref;

use libc::c_int;
use ffi::*;

use super::Encoder;
use ::Error;

pub struct Subtitle(pub Encoder);

impl Subtitle {
	pub fn encode(&self, subtitle: &::Subtitle, out: &mut [u8]) -> Result<bool, Error> {
		unsafe {
			match avcodec_encode_subtitle(self.ptr, out.as_mut_ptr(), out.len() as c_int, &subtitle.val) {
				e if e < 0 => Err(Error::new(e)),
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
