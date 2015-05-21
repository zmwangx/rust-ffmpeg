pub mod side_data;
pub use self::side_data::SideData;

pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

use libc::c_int;
use std::ptr;

use ffi::*;
use ::Dictionary;

bitflags! {
	flags Flags: c_int {
		const FLAG_CORRUPT = AV_FRAME_FLAG_CORRUPT,
	}
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Packet {
	pub duration: i64,
	pub position: i64,
	pub size:     usize,

	pub pts: i64,
	pub dts: i64,
}

#[derive(PartialEq, Eq)]
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
				duration: av_frame_get_pkt_duration(self.ptr) as i64,
				position: av_frame_get_pkt_pos(self.ptr) as i64,
				size:     av_frame_get_pkt_size(self.ptr) as usize,

				pts: (*self.ptr).pkt_pts,
				dts: (*self.ptr).pkt_dts,
			}
		}
	}

	pub fn pts(&self) -> i64 {
		unsafe {
			(*self.ptr).pts as i64
		}
	}

	pub fn best_effort_timestamp(&self) -> i64 {
		unsafe {
			av_frame_get_best_effort_timestamp(self.ptr) as i64
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

unsafe impl Send for Frame { }

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
