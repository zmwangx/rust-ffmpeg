use libc::c_int;
use std::mem;
use std::ops::{Deref, DerefMut};

use ffi::*;
use ::{Rational, Picture};
use ::util::format;
use ::util::chroma;
use ::picture;
use ::color;
use super::Frame;

pub struct Video(Frame);

impl Video {
	pub fn empty() -> Self {
		Video(Frame::new())
	}

	pub fn new(format: format::Pixel, width: u32, height: u32) -> Self {
		unsafe {
			let mut frame = Video(Frame::new());

			frame.set_format(format);
			frame.set_width(width);
			frame.set_height(height);

			av_frame_get_buffer(frame.ptr, 1);

			frame
		}
	}

	pub fn picture(&self) -> Picture {
		Picture::wrap(self.ptr as *mut AVPicture, self.format(), self.width(), self.height())
	}

	pub fn format(&self) -> format::Pixel {
		unsafe {
			if (*self.ptr).format == -1 {
				format::Pixel::None
			}
			else {
				format::Pixel::from(mem::transmute::<_, AVPixelFormat>(((*self.ptr).format)))
			}
		}
	}

	pub fn set_format(&mut self, value: format::Pixel) {
		unsafe {
			(*self.ptr).format = mem::transmute::<AVPixelFormat, c_int>(value.into());
		}
	}

	pub fn kind(&self) -> picture::Type {
		unsafe {
			picture::Type::from((*self.ptr).pict_type)
		}
	}

	pub fn is_interlaced(&self) -> bool {
		unsafe {
			(*self.ptr).interlaced_frame != 0
		}
	}

	pub fn is_top_first(&self) -> bool {
		unsafe {
			(*self.ptr).top_field_first != 0
		}
	}

	pub fn has_palette_changed(&self) -> bool {
		unsafe {
			(*self.ptr).palette_has_changed != 0
		}
	}

	pub fn width(&self) -> u32 {
		unsafe {
			(*self.ptr).width as u32
		}
	}

	pub fn set_width(&mut self, value: u32) {
		unsafe {
			(*self.ptr).width = value as c_int;
		}
	}

	pub fn height(&self) -> u32 {
		unsafe {
			(*self.ptr).height as u32
		}
	}

	pub fn set_height(&mut self, value: u32) {
		unsafe {
			(*self.ptr).height = value as c_int;
		}
	}

	pub fn color_space(&self) -> color::Space {
		unsafe {
			color::Space::from(av_frame_get_colorspace(self.ptr))
		}
	}

	pub fn set_color_space(&mut self, value: color::Space) {
		unsafe {
			av_frame_set_colorspace(self.ptr, value.into());
		}
	}

	pub fn color_range(&self) -> color::Range {
		unsafe {
			color::Range::from(av_frame_get_color_range(self.ptr))
		}
	}

	pub fn set_color_range(&mut self, value: color::Range) {
		unsafe {
			av_frame_set_color_range(self.ptr, value.into());
		}
	}

	pub fn color_primaries(&self) -> color::Primaries {
		unsafe {
			color::Primaries::from((*self.ptr).color_primaries)
		}
	}

	pub fn set_color_primaries(&mut self, value: color::Primaries) {
		unsafe {
			(*self.ptr).color_primaries = value.into();
		}
	}

	pub fn color_transfer_characteristic(&self) -> color::TransferCharacteristic {
		unsafe {
			color::TransferCharacteristic::from((*self.ptr).color_trc)
		}
	}

	pub fn set_color_transfer_characteristic(&mut self, value: color::TransferCharacteristic) {
		unsafe {
			(*self.ptr).color_trc = value.into();
		}
	}

	pub fn chroma_location(&self) -> chroma::Location {
		unsafe {
			chroma::Location::from((*self.ptr).chroma_location)
		}
	}

	pub fn aspect_ratio(&self) -> Rational {
		unsafe {
			Rational((*self.ptr).sample_aspect_ratio)
		}
	}

	pub fn coded_number(&self) -> usize {
		unsafe {
			(*self.ptr).coded_picture_number as usize
		}
	}

	pub fn display_number(&self) -> usize {
		unsafe {
			(*self.ptr).display_picture_number as usize
		}
	}

	pub fn repeat(&self) -> f64 {
		unsafe {
			(*self.ptr).repeat_pict as f64
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
		Video(self.0.clone())
	}

	fn clone_from(&mut self, source: &Self) {
		self.0.clone_from(&source.0);
	}
}

impl Into<Video> for Frame {
	fn into(self) -> Video {
		Video(self)
	}
}

impl Into<Frame> for Video {
	fn into(self) -> Frame {
		self.0
	}
}
