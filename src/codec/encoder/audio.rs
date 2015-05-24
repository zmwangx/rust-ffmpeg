use std::ops::Deref;

use libc::c_int;
use ffi::*;

use super::Encoder;
use ::{Packet, Error};
use ::frame;

pub struct Audio(pub Encoder);

impl Audio {
	pub fn encode(&self, frame: &frame::Audio, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_audio2(self.ptr, &mut out.val, frame.ptr, &mut got) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Audio {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}
