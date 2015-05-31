pub mod side_data;
pub use self::side_data::SideData;

pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

pub mod flag;
pub use self::flag::Flags;

use std::ptr;

use libc::c_int;
use ffi::*;
use ::Dictionary;

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
	pub fn is_key(&self) -> bool {
		unsafe {
			(*self.ptr).key_frame == 1
		}
	}

	pub fn is_corrupt(&self) -> bool {
		self.flags().contains(flag::CORRUPT)
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

	pub fn timestamp(&self) -> Option<i64> {
		unsafe {
			match av_frame_get_best_effort_timestamp(self.ptr) {
				AV_NOPTS_VALUE => None,
				t              => Some(t as i64)
			}
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

impl Drop for Frame {
	fn drop(&mut self) {
		unsafe {
			av_frame_free(&mut self.ptr);
		}
	}
}
