use std::ops::{Deref, DerefMut};

use libc::c_int;
use ffi::*;

use super::Opened;
use ::{packet, Error, AudioService, ChannelLayout};
use ::frame;
use ::util::format;

pub struct Audio(pub Opened);

impl Audio {
	pub fn decode<P: packet::Ref>(&mut self, packet: &P, out: &mut frame::Audio) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_audio4(self.as_mut_ptr(), out.as_mut_ptr(), &mut got, packet.as_ptr()) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}

	pub fn rate(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).sample_rate as u32
		}
	}

	pub fn channels(&self) -> u16 {
		unsafe {
			(*self.as_ptr()).channels as u16
		}
	}

	pub fn format(&self) -> format::Sample {
		unsafe {
			format::Sample::from((*self.as_ptr()).sample_fmt)
		}
	}

	pub fn request_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.as_mut_ptr()).request_sample_fmt = value.into();
		}
	}

	pub fn frames(&self) -> usize {
		unsafe {
			(*self.as_ptr()).frame_number as usize
		}
	}

	pub fn align(&self) -> usize {
		unsafe {
			(*self.as_ptr()).block_align as usize
		}
	}

	pub fn channel_layout(&self) -> ChannelLayout {
		unsafe {
			ChannelLayout::from_bits_truncate((*self.as_ptr()).channel_layout)
		}
	}

	pub fn set_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			(*self.as_mut_ptr()).channel_layout = value.bits();
		}
	}

	pub fn request_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			(*self.as_mut_ptr()).request_channel_layout = value.bits();
		}
	}

	pub fn audio_service(&mut self) -> AudioService {
		unsafe {
			AudioService::from((*self.as_mut_ptr()).audio_service_type)
		}
	}

	pub fn max_bit_rate(&self) -> usize {
		unsafe {
			(*self.as_ptr()).rc_max_rate as usize
		}
	}

	pub fn frame_size(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).frame_size as u32
		}
	}

	pub fn frame_start(&self) -> Option<usize> {
		unsafe {
			match (*self.as_ptr()).timecode_frame_start {
				-1 => None,
				n  => Some(n as usize)
			}
		}
	}
}

impl Deref for Audio {
	type Target = Opened;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Audio {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
