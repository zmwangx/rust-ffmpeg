mod side_data;
pub use self::side_data::SideData;

use std::marker::PhantomData;
use std::mem;
use libc::c_int;

use ffi::*;

bitflags! {
	flags Flags: c_int {
		const FLAG_KEY     = AV_PKT_FLAG_KEY,
		const FLAG_CORRUPT = AV_PKT_FLAG_CORRUPT,
	}
}

pub struct Packet {
	pub val: AVPacket,
}

impl Packet {
	pub fn new() -> Self {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);

			Packet { val: pkt }
		}
	}

	pub fn sized(size: usize) -> Self {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);
			av_new_packet(&mut pkt, size as c_int);

			Packet { val: pkt }
		}
	}

	pub fn shrink(&mut self, size: usize) {
		unsafe {
			av_shrink_packet(&mut self.val, size as c_int);
		}
	}

	pub fn grow(&mut self, size: usize) {
		unsafe {
			av_grow_packet(&mut self.val, size as c_int);
		}
	}

	pub fn flags(&self) -> Flags {
		Flags::from_bits_truncate(self.val.flags)
	}

	pub fn set_flags(&mut self, value: Flags) {
		self.val.flags = value.bits();
	}

	pub fn pts(&self) -> i64 {
		self.val.pts as i64
	}

	pub fn dts(&self) -> i64 {
		self.val.dts as i64
	}

	pub fn size(&self) -> usize {
		self.val.size as usize
	}

	pub fn duration(&self) -> usize {
		self.val.duration as usize
	}

	pub fn position(&self) -> isize {
		self.val.pos as isize
	}

	pub fn convergence(&self) -> isize {
		self.val.convergence_duration as isize
	}

	pub fn side_data(&self) -> SideDataIter {
		SideDataIter::new(&self.val)
	}
}

unsafe impl Send for Packet { }

impl Clone for Packet {
	fn clone(&self) -> Self {
		let mut pkt = Packet::new();
		pkt.clone_from(self);

		pkt
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_copy_packet(&mut self.val, &source.val);
		}
	}
}

impl Drop for Packet {
	fn drop(&mut self) {
		unsafe {
			av_free_packet(&mut self.val);
		}
	}
}

pub struct SideDataIter<'a> {
	ptr: *const AVPacket,
	cur: c_int,

	_marker: PhantomData<&'a Packet>,
}

impl<'a> SideDataIter<'a> {
	pub fn new(ptr: *const AVPacket) -> Self {
		SideDataIter { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> Iterator for SideDataIter<'a> {
	type Item = SideData<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).side_data_elems {
				None
			}
			else {
				self.cur += 1;
				Some(SideData::wrap((*self.ptr).side_data.offset((self.cur - 1) as isize)))
			}
		}
	}
}
