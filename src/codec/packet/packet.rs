use std::marker::PhantomData;
use std::mem;
use std::slice;

use libc::c_int;
use ffi::*;
use ::{Error, Rational, format};
use super::{flag, Flags, SideData, Ref, Mut, Borrow};

pub struct Packet(AVPacket);

unsafe impl Send for Packet { }
unsafe impl Sync for Packet { }

impl Packet {
	#[inline(always)]
	pub unsafe fn is_empty(&self) -> bool {
		self.0.size == 0
	}
}

impl Packet {
	#[inline]
	pub fn empty() -> Self {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);

			Packet(pkt)
		}
	}

	#[inline]
	pub fn new(size: usize) -> Self {
		unsafe {
			let mut pkt: AVPacket = mem::zeroed();

			av_init_packet(&mut pkt);
			av_new_packet(&mut pkt, size as c_int);

			Packet(pkt)
		}
	}

	#[inline]
	pub fn copy(data: &[u8]) -> Self {
		use std::io::Write;

		let mut packet = Packet::new(data.len());
		packet.data_mut().unwrap().write(data).unwrap();

		packet
	}

	#[inline]
	pub fn borrow(data: &[u8]) -> Borrow {
		Borrow::new(data)
	}

	#[inline]
	pub fn shrink(&mut self, size: usize) {
		unsafe {
			av_shrink_packet(&mut self.0, size as c_int);
		}
	}

	#[inline]
	pub fn grow(&mut self, size: usize) {
		unsafe {
			av_grow_packet(&mut self.0, size as c_int);
		}
	}

	#[inline]
	pub fn rescale_ts<S, D>(&mut self, source: S, destination: D)
		where S: Into<Rational>,
		      D: Into<Rational>
	{
		unsafe {
			av_packet_rescale_ts(self.as_mut_ptr(), source.into().into(), destination.into().into());
		}
	}

	#[inline]
	pub fn flags(&self) -> Flags {
		Flags::from_bits_truncate(self.0.flags)
	}

	#[inline]
	pub fn set_flags(&mut self, value: Flags) {
		self.0.flags = value.bits();
	}

	#[inline]
	pub fn is_key(&self) -> bool {
		self.flags().contains(flag::KEY)
	}

	#[inline]
	pub fn is_corrupt(&self) -> bool {
		self.flags().contains(flag::CORRUPT)
	}

	#[inline]
	pub fn stream(&self) -> usize {
		self.0.stream_index as usize
	}

	#[inline]
	pub fn set_stream(&mut self, index: usize) {
		self.0.stream_index = index as c_int;
	}

	#[inline]
	pub fn pts(&self) -> Option<i64> {
		match self.0.pts {
			AV_NOPTS_VALUE => None,
			pts            => Some(pts as i64),
		}
	}

	#[inline]
	pub fn dts(&self) -> i64 {
		self.0.dts as i64
	}

	#[inline]
	pub fn size(&self) -> usize {
		self.0.size as usize
	}

	#[inline]
	pub fn duration(&self) -> i64 {
		self.0.duration as i64
	}

	#[inline]
	pub fn position(&self) -> isize {
		self.0.pos as isize
	}

	#[inline]
	pub fn convergence(&self) -> isize {
		self.0.convergence_duration as isize
	}

	#[inline]
	pub fn side_data(&self) -> SideDataIter {
		SideDataIter::new(&self.0)
	}

	#[inline]
	pub fn data(&self) -> Option<&[u8]> {
		unsafe {
			if self.0.data.is_null() {
				None
			}
			else {
				Some(slice::from_raw_parts(self.0.data, self.0.size as usize))
			}
		}
	}

	#[inline]
	pub fn data_mut(&mut self) -> Option<&mut [u8]> {
		unsafe {
			if self.0.data.is_null() {
				None
			}
			else {
				Some(slice::from_raw_parts_mut(self.0.data, self.0.size as usize))
			}
		}
	}

	#[inline]
	pub fn read(&mut self, format: &mut format::context::Input) -> Result<(), Error> {
		unsafe {
			match av_read_frame(format.as_mut_ptr(), self.as_mut_ptr()) {
				0 => Ok(()),
				e => Err(Error::from(e))
			}
		}
	}

	#[inline]
	pub fn write(&self, format: &mut format::context::Output) -> Result<bool, Error> {
		unsafe {
			if self.is_empty() {
				return Err(Error::InvalidData);
			}

			match av_write_frame(format.as_mut_ptr(), self.as_ptr()) {
				1 => Ok(true),
				0 => Ok(false),
				e => Err(Error::from(e))
			}
		}
	}

	#[inline]
	pub fn write_interleaved(&self, format: &mut format::context::Output) -> Result<bool, Error> {
		unsafe {
			if self.is_empty() {
				return Err(Error::InvalidData);
			}

			match av_interleaved_write_frame(format.as_mut_ptr(), self.as_ptr()) {
				1 => Ok(true),
				0 => Ok(false),
				e => Err(Error::from(e))
			}
		}
	}
}

impl Ref for Packet {
	fn as_ptr(&self) -> *const AVPacket {
		&self.0
	}
}

impl Mut for Packet {
	fn as_mut_ptr(&mut self) -> *mut AVPacket {
		&mut self.0
	}
}

impl Clone for Packet {
	#[inline]
	fn clone(&self) -> Self {
		let mut pkt = Packet::empty();
		pkt.clone_from(self);

		pkt
	}

	#[inline]
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

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			let length = (*self.ptr).side_data_elems as usize;

			(length - self.cur as usize, Some(length - self.cur as usize))
		}
	}
}

impl<'a> ExactSizeIterator for SideDataIter<'a> { }
