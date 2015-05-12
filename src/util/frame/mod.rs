pub mod side_data;
pub use self::side_data::SideData;

use libc::c_int;
use std::ptr;
use std::mem;
use std::ops::Deref;

use ffi::*;
use ::{Dictionary, ColorSpace, ColorRange};
use ::util::pixel_format::PixelFormat;
use ::util::sample_format::SampleFormat;

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

	pub fn metadata<'a>(&'a self) -> Dictionary<'a> {
		unsafe {
			Dictionary::wrap(av_frame_get_metadata(self.ptr))
		}
	}

	pub fn set_metadata(&mut self, mut value: Dictionary) {
		unsafe {
			av_frame_set_metadata(self.ptr, value.take());
		}
	}

	pub fn side_data<'a>(&'a self, kind: side_data::Type) -> Option<SideData<'a>> {
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

	pub fn new_side_data<'a>(&'a mut self, kind: side_data::Type, size: usize) -> Option<SideData<'a>> {
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

	pub fn format(&self) -> SampleFormat {
		unsafe {
			if (*self.ptr).format == -1 {
				SampleFormat::None
			}
			else {
				SampleFormat::from(mem::transmute::<_, AVSampleFormat>(((*self.ptr).format)))
			}
		}
	}

	pub fn channel_layout(&self) -> i64 {
		unsafe {
			av_frame_get_channel_layout(self.0.ptr)
		}
	}

	pub fn set_channel_layout(&mut self, value: i64) {
		unsafe {
			av_frame_set_channel_layout(self.0.ptr, value);
		}
	}

	pub fn channels(&self) -> usize {
		unsafe {
			av_frame_get_channels(self.0.ptr) as usize
		}
	}

	pub fn set_channels(&mut self, value: usize) {
		unsafe {
			av_frame_set_channels(self.0.ptr, value as c_int);
		}
	}

	pub fn sample_rate(&self) -> i32 {
		unsafe {
			av_frame_get_sample_rate(self.0.ptr)
		}
	}

	pub fn set_sample_rate(&mut self, value: i32) {
		unsafe {
			av_frame_set_sample_rate(self.0.ptr, value);
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

	pub fn format(&self) -> PixelFormat {
		unsafe {
			if (*self.ptr).format == -1 {
				PixelFormat::None
			}
			else {
				PixelFormat::from(mem::transmute::<_, AVPixelFormat>(((*self.ptr).format)))
			}
		}
	}

	pub fn width(&self) -> usize {
		unsafe {
			(*self.0.ptr).width as usize
		}
	}

	pub fn height(&self) -> usize {
		unsafe {
			(*self.0.ptr).height as usize
		}
	}

	pub fn color_space(&self) -> ColorSpace {
		unsafe {
			ColorSpace::from(av_frame_get_colorspace(self.0.ptr))
		}
	}

	pub fn set_color_space(&mut self, value: ColorSpace) {
		unsafe {
			av_frame_set_colorspace(self.0.ptr, value.into());
		}
	}

	pub fn color_range(&self) -> ColorRange {
		unsafe {
			ColorRange::from(av_frame_get_color_range(self.0.ptr))
		}
	}

	pub fn set_color_range(&mut self, value: ColorRange) {
		unsafe {
			av_frame_set_color_range(self.0.ptr, value.into());
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
