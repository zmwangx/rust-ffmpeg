use std::ops::Deref;

use libc::c_int;
use ffi::*;

use super::Encoder;
use ::{Packet, Error};
use ::frame;

pub struct Video(pub Encoder);

impl Video {
	pub fn encode(&self, frame: &frame::Video, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_video2(self.ptr, &mut out.val, frame.ptr, &mut got) {
				e if e < 0 => Err(Error::new(e)),
				_          => Ok(got != 0)
			}
		}
	}
}

impl Deref for Video {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}
