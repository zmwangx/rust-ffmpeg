use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ffi::CString;

use ffi::*;
use ::{Error, Codec, Stream, Packet, format};
use super::common::Context;

pub struct Input {
	ptr: *mut AVFormatContext,
	ctx: Context,
}

unsafe impl Send for Input { }

impl Input {
	pub unsafe fn wrap(ptr: *mut AVFormatContext) -> Self {
		Input { ptr: ptr, ctx: Context::wrap(ptr) }
	}

	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}
}

impl Input {
	pub fn format(&self) -> format::Input {
		unsafe {
			format::Input::wrap((*self.as_ptr()).iformat)
		}
	}

	pub fn video_codec(&self) -> Option<Codec> {
		unsafe {
			let ptr = av_format_get_video_codec(self.as_ptr());

			if ptr.is_null() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_video_codec(&mut self, mut value: Codec) {
		unsafe {
			av_format_set_video_codec(self.as_mut_ptr(), value.as_mut_ptr());
		}
	}

	pub fn audio_codec(&self) -> Option<Codec> {
		unsafe {
			let ptr = av_format_get_audio_codec(self.as_ptr());

			if ptr.is_null() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_audio_codec(&mut self, mut value: Codec) {
		unsafe {
			av_format_set_audio_codec(self.as_mut_ptr(), value.as_mut_ptr());
		}
	}

	pub fn subtitle_codec(&self) -> Option<Codec> {
		unsafe {
			let ptr = av_format_get_subtitle_codec(self.as_ptr());

			if ptr.is_null() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_subtitle_codec(&mut self, mut value: Codec) {
		unsafe {
			av_format_set_subtitle_codec(self.as_mut_ptr(), value.as_mut_ptr());
		}
	}

	pub fn data_codec(&self) -> Option<Codec> {
		unsafe {
			let ptr = av_format_get_data_codec(self.as_ptr());

			if ptr.is_null() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_data_codec(&mut self, mut value: Codec) {
		unsafe {
			av_format_set_data_codec(self.as_mut_ptr(), value.as_mut_ptr());
		}
	}

	pub fn probe_score(&self) -> i32 {
		unsafe {
			av_format_get_probe_score(self.as_ptr())
		}
	}

	pub fn packets(&mut self) -> PacketIter {
		PacketIter::new(self)
	}
}

impl Deref for Input {
	type Target = Context;

	fn deref(&self) -> &Self::Target {
		&self.ctx
	}
}

impl DerefMut for Input {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.ctx
	}
}

impl Drop for Input {
	fn drop(&mut self) {
		unsafe {
			avformat_close_input(&mut self.as_mut_ptr());
		}
	}
}

pub struct PacketIter<'a> {
	context: &'a mut Input,
}

impl<'a> PacketIter<'a> {
	pub fn new(context: &mut Input) -> PacketIter {
		PacketIter { context: context }
	}
}

impl<'a> Iterator for PacketIter<'a> {
	type Item = (Stream<'a>, Packet);

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		let mut packet = Packet::empty();

		loop {
			match packet.read(self.context) {
				Ok(..) =>
					return Some((unsafe {
						Stream::wrap(*(*self.context.as_ptr()).streams.offset(packet.stream() as isize))
					}, packet)),

				Err(Error::Eof) =>
					return None,

				Err(..) =>
					()
			}
		}
	}
}

pub fn dump(ctx: &Input, index: i32, url: Option<&str>) {
	let url = url.map(|u| CString::new(u).unwrap());

	unsafe {
		av_dump_format(ctx.as_ptr(), index,
			url.map(|u| u.as_ptr()).unwrap_or(ptr::null()), 0);
	}
}
