pub mod side_data;
pub use self::side_data::SideData;

pub mod flag;
pub use self::flag::Flags;

use std::marker::PhantomData;
use std::mem;

use libc::c_int;
use ffi::*;
use ::{Error, format};

pub struct Packet(AVPacket);

unsafe impl Send for Packet { }

impl Packet {
	pub unsafe fn as_ptr(&self) -> *const AVPacket {
		&self.0
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVPacket {
		&mut self.0
	}
}

impl Packet {
	pub fn empty() -> Self {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);

			Packet(pkt)
		}
	}

	pub fn new(size: usize) -> Self {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);
			av_new_packet(&mut pkt, size as c_int);

			Packet(pkt)
		}
	}

	pub fn shrink(&mut self, size: usize) {
		unsafe {
			av_shrink_packet(&mut self.0, size as c_int);
		}
	}

	pub fn grow(&mut self, size: usize) {
		unsafe {
			av_grow_packet(&mut self.0, size as c_int);
		}
	}

	pub fn flags(&self) -> Flags {
		Flags::from_bits_truncate(self.0.flags)
	}

	pub fn set_flags(&mut self, value: Flags) {
		self.0.flags = value.bits();
	}

	pub fn is_key(&self) -> bool {
		self.flags().contains(flag::KEY)
	}

	pub fn is_corrupt(&self) -> bool {
		self.flags().contains(flag::CORRUPT)
	}

	pub fn stream(&self) -> usize {
		self.0.stream_index as usize
	}

	pub fn set_stream(&mut self, index: usize) {
		self.0.stream_index = index as c_int;
	}

	pub fn pts(&self) -> i64 {
		self.0.pts as i64
	}

	pub fn dts(&self) -> i64 {
		self.0.dts as i64
	}

	pub fn size(&self) -> usize {
		self.0.size as usize
	}

	pub fn duration(&self) -> usize {
		self.0.duration as usize
	}

	pub fn position(&self) -> isize {
		self.0.pos as isize
	}

	pub fn convergence(&self) -> isize {
		self.0.convergence_duration as isize
	}

	pub fn side_data(&self) -> SideDataIter {
		SideDataIter::new(&self.0)
	}

	pub fn read(&mut self, format: &mut format::Context) -> Result<(), Error> {
		unsafe {
			match av_read_frame(format.as_mut_ptr(), self.as_mut_ptr()) {
				0 => Ok(()),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn write(&self, format: &mut format::Context) -> Result<bool, Error> {
		unsafe {
			match av_write_frame(format.as_mut_ptr(), self.as_ptr()) {
				1 => Ok(true),
				0 => Ok(false),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn write_interleaved(&self, format: &mut format::Context) -> Result<bool, Error> {
		unsafe {
			match av_interleaved_write_frame(format.as_mut_ptr(), self.as_ptr()) {
				1 => Ok(true),
				0 => Ok(false),
				e => Err(Error::from(e))
			}
		}
	}
}

impl Clone for Packet {
	fn clone(&self) -> Self {
		let mut pkt = Packet::empty();
		pkt.clone_from(self);

		pkt
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_copy_packet(&mut self.0, &source.0);
		}
	}
}

impl Drop for Packet {
	fn drop(&mut self) {
		unsafe {
			av_free_packet(&mut self.0);
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
