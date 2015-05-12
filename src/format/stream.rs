use std::marker::PhantomData;

use libc::c_int;
use ffi::*;
use ::format;
use ::codec::{self, packet};
use ::{Rational, Discard};

bitflags! {
	flags Disposition: i32 {
		const DISPOSITION_DEFAULT          = AV_DISPOSITION_DEFAULT,
		const DISPOSITION_DUB              = AV_DISPOSITION_DUB,
		const DISPOSITION_ORIGINAL         = AV_DISPOSITION_ORIGINAL,
		const DISPOSITION_COMMENT          = AV_DISPOSITION_COMMENT,
		const DISPOSITION_LYRICS           = AV_DISPOSITION_LYRICS,
		const DISPOSITION_KARAOKE          = AV_DISPOSITION_KARAOKE,
		const DISPOSITION_FORCED           = AV_DISPOSITION_FORCED,
		const DISPOSITION_HEARING_IMPAIRED = AV_DISPOSITION_HEARING_IMPAIRED,
		const DISPOSITION_VISUAL_IMPAIRED  = AV_DISPOSITION_VISUAL_IMPAIRED,
		const DISPOSITION_CLEAN_EFFECTS    = AV_DISPOSITION_CLEAN_EFFECTS,
		const DISPOSITION_ATTACHED_PIC     = AV_DISPOSITION_ATTACHED_PIC,
		const DISPOSITION_CAPTIONS         = AV_DISPOSITION_CAPTIONS,
		const DISPOSITION_DESCRIPTIONS     = AV_DISPOSITION_DESCRIPTIONS,
		const DISPOSITION_METADATA         = AV_DISPOSITION_METADATA,
	}
}

#[derive(Eq, PartialEq)]
pub struct Stream<'a> {
	ptr: *mut AVStream,

	_marker: PhantomData<&'a format::Context>,
}

impl<'a> Stream<'a> {
	pub fn wrap(ptr: *mut AVStream) -> Self {
		Stream { ptr: ptr, _marker: PhantomData }
	}

	pub fn codec(&self) -> codec::Context {
		unsafe {
			codec::Context::wrap((*self.ptr).codec)
		}
	}

	pub fn index(&self) -> usize {
		unsafe {
			(*self.ptr).index as usize
		}
	}

	pub fn time_base(&self) -> Rational {
		unsafe {
			Rational((*self.ptr).time_base)
		}
	}

	pub fn start_time(&self) -> i64 {
		unsafe {
			(*self.ptr).start_time
		}
	}

	pub fn duration(&self) -> i64 {
		unsafe {
			(*self.ptr).duration
		}
	}

	pub fn frames(&self) -> i64 {
		unsafe {
			(*self.ptr).nb_frames
		}
	}

	pub fn disposition(&self) -> Disposition {
		unsafe {
			Disposition::from_bits_truncate((*self.ptr).disposition)
		}
	}

	pub fn discard(&self) -> Discard {
		unsafe {
			Discard::from((*self.ptr).discard)
		}
	}

	pub fn side_data(&self) -> SideDataIter {
		SideDataIter::new(self.ptr)
	}

	pub fn frame_rate(&self) -> Rational {
		unsafe {
			Rational(av_stream_get_r_frame_rate(self.ptr))
		}
	}

	pub fn set_frame_rate(&self, value: Rational) {
		unsafe {
			av_stream_set_r_frame_rate(self.ptr, value.0);
		}
	}
}

pub struct SideDataIter<'a> {
	ptr: *mut AVStream,
	cur: c_int,

	_marker: PhantomData<&'a Stream<'a>>,
}

impl<'a> SideDataIter<'a> {
	pub fn new(ptr: *mut AVStream) -> Self {
		SideDataIter { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> Iterator for SideDataIter<'a> {
	type Item = packet::SideData<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).nb_side_data {
				None
			}
			else {
				self.cur += 1;
				Some(packet::SideData::wrap((*self.ptr).side_data.offset((self.cur - 1) as isize)))
			}
		}
	}
}
