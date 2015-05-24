use std::ops::Deref;

use libc::c_int;
use ffi::*;

use super::Decoder;
use ::{Packet, Error, AudioService};
use ::frame;
use ::util::format;

pub struct Audio(pub Decoder);

impl Audio {
	pub fn decode(&mut self, packet: &Packet, out: &mut frame::Audio) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_audio4(self.ptr, out.ptr, &mut got, &packet.val) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}

	pub fn rate(&self) -> i32 {
		unsafe {
			(*self.ptr).sample_rate as i32
		}
	}

	pub fn channels(&self) -> usize {
		unsafe {
			(*self.ptr).channels as usize
		}
	}

	pub fn format(&self) -> format::Sample {
		unsafe {
			format::Sample::from((*self.ptr).sample_fmt)
		}
	}

	pub fn request_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.ptr).request_sample_fmt = value.into();
		}
	}

	pub fn frames(&self) -> usize {
		unsafe {
			(*self.ptr).frame_number as usize
		}
	}

	pub fn align(&self) -> usize {
		unsafe {
			(*self.ptr).block_align as usize
		}
	}

	pub fn channel_layout(&self) -> u64 {
		unsafe {
			(*self.ptr).channel_layout
		}
	}

	pub fn set_channel_layout(&mut self, value: u64) {
		unsafe {
			(*self.ptr).channel_layout = value;
		}
	}

	pub fn request_channel_layout(&mut self, value: u64) {
		unsafe {
			(*self.ptr).request_channel_layout = value;
		}
	}

	pub fn audio_service(&mut self) -> AudioService {
		unsafe {
			AudioService::from((*self.ptr).audio_service_type)
		}
	}

	pub fn max_rate(&self) -> usize {
		unsafe {
			(*self.ptr).rc_max_rate as usize
		}
	}

	pub fn frame_start(&self) -> Option<usize> {
		unsafe {
			match (*self.ptr).timecode_frame_start {
				-1 => None,
				n  => Some(n as usize)
			}
		}
	}
}

impl Deref for Audio {
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}
