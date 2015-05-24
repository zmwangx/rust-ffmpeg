use std::ops::Deref;

use libc::c_int;
use ffi::*;

use super::{Decoder, slice};
use ::{Packet, Error, Rational, FieldOrder};
use ::frame;
use ::util::format;
use ::util::chroma;
use ::color;

pub struct Video(pub Decoder);

impl Video {
	pub fn decode(&mut self, packet: &Packet, out: &mut frame::Video) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_decode_video2(self.ptr, out.ptr, &mut got, &packet.val) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}

	pub fn width(&self) -> u32 {
		unsafe {
			(*self.ptr).width as u32
		}
	}

	pub fn height(&self) -> u32 {
		unsafe {
			(*self.ptr).height as u32
		}
	}

	pub fn format(&self) -> format::Pixel {
		unsafe {
			format::Pixel::from((*self.ptr).pix_fmt)
		}
	}

	pub fn set_format(&mut self, value: format::Pixel) {
		unsafe {
			(*self.ptr).pix_fmt = value.into();
		}
	}

	pub fn has_b_frames(&self) -> bool {
		unsafe {
			(*self.ptr).has_b_frames != 0
		}
	}

	pub fn aspect_ratio(&self) -> Rational {
		unsafe {
			Rational((*self.ptr).sample_aspect_ratio)
		}
	}

	pub fn color_space(&self) -> color::Space {
		unsafe {
			color::Space::from((*self.ptr).colorspace)
		}
	}

	pub fn color_range(&self) -> color::Range {
		unsafe {
			color::Range::from((*self.ptr).color_range)
		}
	}

	pub fn color_primaries(&self) -> color::Primaries {
		unsafe {
			color::Primaries::from((*self.ptr).color_primaries)
		}
	}

	pub fn color_transfer_characteristic(&self) -> color::TransferCharacteristic {
		unsafe {
			color::TransferCharacteristic::from((*self.ptr).color_trc)
		}
	}

	pub fn chroma_location(&self) -> chroma::Location {
		unsafe {
			chroma::Location::from((*self.ptr).chroma_sample_location)
		}
	}

	pub fn set_slice_count(&mut self, value: usize) {
		unsafe {
			(*self.ptr).slice_count = value as c_int;
		}
	}

	pub fn set_slice_flags(&mut self, value: slice::Flags) {
		unsafe {
			(*self.ptr).slice_flags = value.bits();
		}
	}

	pub fn skip_top(&mut self, value: usize) {
		unsafe {
			(*self.ptr).skip_top = value as c_int;
		}
	}

	pub fn skip_bottom(&mut self, value: usize) {
		unsafe {
			(*self.ptr).skip_bottom = value as c_int;
		}
	}

	pub fn references(&self) -> usize {
		unsafe {
			(*self.ptr).refs as usize
		}
	}

	pub fn set_field_order(&mut self, value: FieldOrder) {
		unsafe {
			(*self.ptr).field_order = value.into();
		}
	}

	// intra_matrix
	// inter_matrix
}

impl Deref for Video {
	type Target = Decoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}


