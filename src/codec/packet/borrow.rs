use std::mem;
use std::ptr;

use libc::c_int;
use ffi::*;
use super::Ref;

pub struct Borrow<'a> {
	packet: AVPacket,
	data:   &'a [u8],
}

impl<'a> Borrow<'a> {
	pub fn new(data: &[u8]) -> Borrow {
		unsafe {
			let mut packet: AVPacket = mem::zeroed();

			packet.data = data.as_ptr() as *mut _;
			packet.size = data.len() as c_int;

			Borrow {
				packet: packet,
				data:   data,
			}
		}
	}

	#[inline]
	pub fn size(&self) -> usize {
		self.packet.size as usize
	}

	#[inline]
	pub fn data(&self) -> Option<&[u8]> {
		Some(self.data)
	}
}

impl<'a> Ref for Borrow<'a> {
	fn as_ptr(&self) -> *const AVPacket {
		&self.packet
	}
}

impl<'a> Drop for Borrow<'a> {
	fn drop(&mut self) {
		unsafe {
			self.packet.data = ptr::null_mut();
			self.packet.size = 0;

			av_free_packet(&mut self.packet);
		}
	}
}
