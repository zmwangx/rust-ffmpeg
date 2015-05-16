pub mod side_data;
pub use self::side_data::SideData;

use libc::c_int;
use std::ptr;
use std::mem;
use std::ops::Deref;

use ffi::*;
use ::{Dictionary, Rational, Picture};
use ::util::format;
use ::util::chroma;
use ::picture;
use ::color;

bitflags! {
	flags Flags: c_int {
		const FLAG_CORRUPT = AV_FRAME_FLAG_CORRUPT,
	}
}

pub struct Packet {
	pub duration: isize,
	pub position: isize,
	pub size:     usize,
}

pub struct Frame {
	pub ptr: *mut AVFrame,
}

impl Frame {
	pub fn new() -> Self {
		unsafe {
			Frame { ptr: av_frame_alloc() }
		}
	}

	pub fn make_unique(&mut self) -> &mut Self {
		unsafe {
			av_frame_make_writable(self.ptr);
		}

		self
	}

	pub fn is_unique(&self) -> bool {
		unsafe {
			av_frame_is_writable(self.ptr) == 0
		}
	}

	pub fn is_key(&self) -> bool {
		unsafe {
			(*self.ptr).key_frame == 1
		}
	}

	pub fn is_corrupt(&self) -> bool {
		self.flags().contains(FLAG_CORRUPT)
	}

	pub fn packet(&self) -> Packet {
		unsafe {
			Packet {
				duration: av_frame_get_pkt_duration(self.ptr) as isize,
				position: av_frame_get_pkt_pos(self.ptr) as isize,
				size:     av_frame_get_pkt_size(self.ptr) as usize,
			}
		}
	}

	pub fn best_effort_timestamp(&self) -> isize {
		unsafe {
			av_frame_get_best_effort_timestamp(self.ptr) as isize
		}
	}

	pub fn quality(&self) -> usize {
		unsafe {
			(*self.ptr).quality as usize
		}
	}

	pub fn flags(&self) -> Flags {
		unsafe {
			Flags::from_bits_truncate((*self.ptr).flags)
		}
	}

	pub fn metadata(&self) -> Dictionary {
		unsafe {
			Dictionary::wrap(av_frame_get_metadata(self.ptr))
		}
	}

	pub fn set_metadata(&mut self, mut value: Dictionary) {
		unsafe {
			av_frame_set_metadata(self.ptr, value.take());
		}
	}

	pub fn side_data(&self, kind: side_data::Type) -> Option<SideData> {
		unsafe {
			let ptr = av_frame_get_side_data(self.ptr, kind.into());

			if ptr == ptr::null_mut() {
				None
			}
			else {
				Some(SideData::wrap(ptr))
			}
		}
	}

	pub fn new_side_data(&mut self, kind: side_data::Type, size: usize) -> Option<SideData> {
		unsafe {
			let ptr = av_frame_new_side_data(self.ptr, kind.into(), size as c_int);

			if ptr == ptr::null_mut() {
				None
			}
			else {
				Some(SideData::wrap(ptr))
			}
		}
	}

	pub fn remove_side_data(&mut self, kind: side_data::Type) {
		unsafe {
			av_frame_remove_side_data(self.ptr, kind.into());
		}
	}
}

impl Clone for Frame {
	fn clone(&self) -> Self {
		unsafe {
			Frame { ptr: av_frame_clone(self.ptr) }
		}
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.ptr, source.ptr);
			av_frame_copy_props(self.ptr, source.ptr);
		}
	}
}

impl Drop for Frame {
	fn drop(&mut self) {
		unsafe {
			av_frame_free(&mut self.ptr);
		}
	}
}

pub struct Audio(Frame);

impl Audio {
	pub fn new() -> Self {
		Audio(Frame::new())
	}

	pub fn format(&self) -> format::Sample {
		unsafe {
			if (*self.ptr).format == -1 {
				format::Sample::None
			}
			else {
				format::Sample::from(mem::transmute::<_, AVSampleFormat>(((*self.ptr).format)))
			}
		}
	}

	pub fn channel_layout(&self) -> i64 {
		unsafe {
			av_frame_get_channel_layout(self.ptr)
		}
	}

	pub fn set_channel_layout(&mut self, value: i64) {
		unsafe {
			av_frame_set_channel_layout(self.ptr, value);
		}
	}

	pub fn channels(&self) -> usize {
		unsafe {
			av_frame_get_channels(self.ptr) as usize
		}
	}

	pub fn set_channels(&mut self, value: usize) {
		unsafe {
			av_frame_set_channels(self.ptr, value as c_int);
		}
	}

	pub fn rate(&self) -> i32 {
		unsafe {
			av_frame_get_sample_rate(self.ptr)
		}
	}

	pub fn set_rate(&mut self, value: i32) {
		unsafe {
			av_frame_set_sample_rate(self.ptr, value);
		}
	}

	pub fn samples(&self) -> usize {
		unsafe {
			(*self.ptr).nb_samples as usize
		}
	}

	pub fn set_samples(&mut self, value: usize) {
		unsafe {
			(*self.ptr).nb_samples = value as c_int;
		}
	}
}

impl Deref for Audio {
	type Target = Frame;

	fn deref(&self) -> &Frame {
		&self.0
	}
}

impl Into<Frame> for Audio {
	fn into(self) -> Frame {
		self.0
	}
}

impl Into<Audio> for Frame {
	fn into(self) -> Audio {
		Audio(self)
	}
}

pub struct Video(Frame);

impl Deref for Video {
	type Target = Frame;

	fn deref(&self) -> &Frame {
		&self.0
	}
}

impl Video {
	pub fn new() -> Self {
		Video(Frame::new())
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

	pub fn width(&self) -> usize {
		unsafe {
			(*self.ptr).width as usize
		}
	}

	pub fn set_width(&mut self, value: usize) {
		unsafe {
			(*self.ptr).width = value as c_int;
		}
	}

	pub fn height(&self) -> usize {
		unsafe {
			(*self.ptr).height as usize
		}
	}

	pub fn set_height(&mut self, value: usize) {
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
