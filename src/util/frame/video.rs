use libc::c_int;
use std::mem;
use std::ops::{Deref, DerefMut};

use ffi::*;
use ::Rational;
use ::util::format;
use ::util::chroma;
use ::picture;
use ::color;
use super::Frame;

#[derive(PartialEq, Eq)]
pub struct Video(Frame);

impl Video {
	pub unsafe fn wrap(ptr: *mut AVFrame) -> Self {
		Video(Frame::wrap(ptr))
	}
}

impl Video {
	pub fn empty() -> Self {
		unsafe {
			Video(Frame::empty())
		}
	}

	pub fn new(format: format::Pixel, width: u32, height: u32) -> Self {
		unsafe {
			let mut frame = Video::empty();

			frame.set_format(format);
			frame.set_width(width);
			frame.set_height(height);

			av_frame_get_buffer(frame.as_mut_ptr(), 1);

			frame
		}
	}

	pub fn format(&self) -> format::Pixel {
		unsafe {
			if (*self.as_ptr()).format == -1 {
				format::Pixel::None
			}
			else {
				format::Pixel::from(mem::transmute::<_, AVPixelFormat>(((*self.as_ptr()).format)))
			}
		}
	}

	pub fn set_format(&mut self, value: format::Pixel) {
		unsafe {
			(*self.as_mut_ptr()).format = mem::transmute::<AVPixelFormat, c_int>(value.into());
		}
	}

	pub fn kind(&self) -> picture::Type {
		unsafe {
			picture::Type::from((*self.as_ptr()).pict_type)
		}
	}

	pub fn is_interlaced(&self) -> bool {
		unsafe {
			(*self.as_ptr()).interlaced_frame != 0
		}
	}

	pub fn is_top_first(&self) -> bool {
		unsafe {
			(*self.as_ptr()).top_field_first != 0
		}
	}

	pub fn has_palette_changed(&self) -> bool {
		unsafe {
			(*self.as_ptr()).palette_has_changed != 0
		}
	}

	pub fn width(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).width as u32
		}
	}

	pub fn set_width(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).width = value as c_int;
		}
	}

	pub fn height(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).height as u32
		}
	}

	pub fn set_height(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).height = value as c_int;
		}
	}

	pub fn color_space(&self) -> color::Space {
		unsafe {
			color::Space::from(av_frame_get_colorspace(self.as_ptr()))
		}
	}

	pub fn set_color_space(&mut self, value: color::Space) {
		unsafe {
			av_frame_set_colorspace(self.as_mut_ptr(), value.into());
		}
	}

	pub fn color_range(&self) -> color::Range {
		unsafe {
			color::Range::from(av_frame_get_color_range(self.as_ptr()))
		}
	}

	pub fn set_color_range(&mut self, value: color::Range) {
		unsafe {
			av_frame_set_color_range(self.as_mut_ptr(), value.into());
		}
	}

	pub fn color_primaries(&self) -> color::Primaries {
		unsafe {
			color::Primaries::from((*self.as_ptr()).color_primaries)
		}
	}

	pub fn set_color_primaries(&mut self, value: color::Primaries) {
		unsafe {
			(*self.as_mut_ptr()).color_primaries = value.into();
		}
	}

	pub fn color_transfer_characteristic(&self) -> color::TransferCharacteristic {
		unsafe {
			color::TransferCharacteristic::from((*self.as_ptr()).color_trc)
		}
	}

	pub fn set_color_transfer_characteristic(&mut self, value: color::TransferCharacteristic) {
		unsafe {
			(*self.as_mut_ptr()).color_trc = value.into();
		}
	}

	pub fn chroma_location(&self) -> chroma::Location {
		unsafe {
			chroma::Location::from((*self.as_ptr()).chroma_location)
		}
	}

	pub fn aspect_ratio(&self) -> Rational {
		unsafe {
			Rational((*self.as_ptr()).sample_aspect_ratio)
		}
	}

	pub fn coded_number(&self) -> usize {
		unsafe {
			(*self.as_ptr()).coded_picture_number as usize
		}
	}

	pub fn display_number(&self) -> usize {
		unsafe {
			(*self.as_ptr()).display_picture_number as usize
		}
	}

	pub fn repeat(&self) -> f64 {
		unsafe {
			(*self.as_ptr()).repeat_pict as f64
		}
	}
}

unsafe impl Send for Video { }

impl Deref for Video {
	type Target = Frame;

	fn deref(&self) -> &Frame {
		&self.0
	}
}

impl DerefMut for Video {
	fn deref_mut(&mut self) -> &mut Frame {
		&mut self.0
	}
}

impl Clone for Video {
	fn clone(&self) -> Self {
		let mut cloned = Video::new(self.format(), self.width(), self.height());
		cloned.clone_from(self);

		cloned
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.as_mut_ptr(), source.as_ptr());
			av_frame_copy_props(self.as_mut_ptr(), source.as_ptr());
		}
	}
}
