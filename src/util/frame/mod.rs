pub mod side_data;
pub use self::side_data::SideData;

pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

pub mod flag;
pub use self::flag::Flags;

use libc::c_int;
use ffi::*;
use ::{Dictionary, DictionaryRef};

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
	ptr: *mut AVFrame,

	_own: bool,
}

unsafe impl Send for Frame { }

impl Frame {
	pub unsafe fn wrap(ptr: *mut AVFrame) -> Self {
		Frame { ptr: ptr, _own: false }
	}

	pub unsafe fn empty() -> Self {
		Frame { ptr: av_frame_alloc(), _own: true }
	}

	pub unsafe fn as_ptr(&self) -> *const AVFrame {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFrame {
		self.ptr
	}

	pub unsafe fn is_empty(&self) -> bool {
		(*self.as_ptr()).data[0].is_null()
	}
}

impl Frame {
	pub fn is_key(&self) -> bool {
		unsafe {
			(*self.as_ptr()).key_frame == 1
		}
	}

	pub fn is_corrupt(&self) -> bool {
		self.flags().contains(flag::CORRUPT)
	}

	pub fn packet(&self) -> Packet {
		unsafe {
			Packet {
				duration: av_frame_get_pkt_duration(self.as_ptr()) as i64,
				position: av_frame_get_pkt_pos(self.as_ptr()) as i64,
				size:     av_frame_get_pkt_size(self.as_ptr()) as usize,

				pts: (*self.as_ptr()).pkt_pts,
				dts: (*self.as_ptr()).pkt_dts,
			}
		}
	}

	pub fn pts(&self) -> i64 {
		unsafe {
			(*self.as_ptr()).pts as i64
		}
	}

	pub fn set_pts(&mut self, value: i64) {
		unsafe {
			(*self.as_mut_ptr()).pts = value;
		}
	}

	pub fn timestamp(&self) -> Option<i64> {
		unsafe {
			match av_frame_get_best_effort_timestamp(self.as_ptr()) {
				AV_NOPTS_VALUE => None,
				t              => Some(t as i64)
			}
		}
	}

	pub fn quality(&self) -> usize {
		unsafe {
			(*self.as_ptr()).quality as usize
		}
	}

	pub fn flags(&self) -> Flags {
		unsafe {
			Flags::from_bits_truncate((*self.as_ptr()).flags)
		}
	}

	pub fn metadata(&self) -> DictionaryRef {
		unsafe {
			DictionaryRef::wrap(av_frame_get_metadata(self.as_ptr()))
		}
	}

	pub fn set_metadata(&mut self, value: Dictionary) {
		unsafe {
			av_frame_set_metadata(self.as_mut_ptr(), value.disown());
		}
	}

	pub fn side_data(&self, kind: side_data::Type) -> Option<SideData> {
		unsafe {
			let ptr = av_frame_get_side_data(self.as_ptr(), kind.into());

			if ptr.is_null() {
				None
			}
			else {
				Some(SideData::wrap(ptr))
			}
		}
	}

	pub fn new_side_data(&mut self, kind: side_data::Type, size: usize) -> Option<SideData> {
		unsafe {
			let ptr = av_frame_new_side_data(self.as_mut_ptr(), kind.into(), size as c_int);

			if ptr.is_null() {
				None
			}
			else {
				Some(SideData::wrap(ptr))
			}
		}
	}

	pub fn remove_side_data(&mut self, kind: side_data::Type) {
		unsafe {
			av_frame_remove_side_data(self.as_mut_ptr(), kind.into());
		}
	}
}

impl Drop for Frame {
	fn drop(&mut self) {
		unsafe {
			av_frame_free(&mut self.as_mut_ptr());
		}
	}
}
