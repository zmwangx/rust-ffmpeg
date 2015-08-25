use std::marker::PhantomData;

use libc::c_uint;
use ffi::*;
use ::{Error, Codec, Stream, StreamMut, Packet, Dictionary};

pub struct Context {
	ptr: *mut AVFormatContext,

	_input: bool,
}

unsafe impl Send for Context { }

impl Context {
	pub unsafe fn input(ptr: *mut AVFormatContext) -> Self {
		Context {
			ptr: ptr,

			_input: true,
		}
	}

	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}
}

impl Context {
	pub fn new() -> Self {
		unsafe {
			Context {
				ptr: avformat_alloc_context(),

				_input: false,
			}
		}
	}

	pub fn is_input(&self) -> bool {
		self._input
	}

	pub fn is_output(&self) -> bool {
		!self._input
	}

	pub fn stream(&self, index: usize) -> Option<Stream> {
		unsafe {
			if index >= (*self.as_ptr()).nb_streams as usize {
				None
			}
			else {
				Some(Stream::wrap(*(*self.ptr).streams.offset(index as isize)))
			}
		}
	}

	pub fn stream_mut(&mut self, index: usize) -> Option<StreamMut> {
		unsafe {
			if index >= (*self.as_ptr()).nb_streams as usize {
				None
			}
			else {
				Some(StreamMut::wrap(*(*self.ptr).streams.offset(index as isize)))
			}
		}
	}

	pub fn streams(&self) -> StreamIter {
		unsafe {
			StreamIter::new(self.as_ptr())
		}
	}

	pub fn streams_mut(&mut self) -> StreamIterMut {
		unsafe {
			StreamIterMut::new(self.as_mut_ptr())
		}
	}

	pub fn metadata(&self) -> Dictionary {
		unsafe {
			Dictionary::wrap((*self.as_ptr()).metadata)
		}
	}

	pub fn probe_score(&self) -> i32 {
		unsafe {
			av_format_get_probe_score(self.as_ptr())
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

	pub fn packets(&mut self) -> PacketIter {
		PacketIter::new(self)
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			if self._input {
				avformat_close_input(&mut self.as_mut_ptr());
			}
			else {
				avformat_free_context(self.as_mut_ptr());
			}
		}
	}
}

pub struct StreamIter<'a> {
	ptr: *const AVFormatContext,
	cur: c_uint,

	_marker: PhantomData<&'a Context>,
}

impl<'a> StreamIter<'a> {
	pub fn new(ptr: *const AVFormatContext) -> Self {
		StreamIter { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> Iterator for StreamIter<'a> {
	type Item = Stream<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).nb_streams {
				None
			}
			else {
				self.cur += 1;
				Some(Stream::wrap(*(*self.ptr).streams.offset((self.cur - 1) as isize)))
			}
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			((*self.ptr).nb_streams as usize, Some((*self.ptr).nb_streams as usize))
		}
	}
}

impl<'a> ExactSizeIterator for StreamIter<'a> { }

pub struct StreamIterMut<'a> {
	ptr: *const AVFormatContext,
	cur: c_uint,

	_marker: PhantomData<&'a Context>,
}

impl<'a> StreamIterMut<'a> {
	pub fn new(ptr: *mut AVFormatContext) -> Self {
		StreamIterMut { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> Iterator for StreamIterMut<'a> {
	type Item = StreamMut<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).nb_streams {
				None
			}
			else {
				self.cur += 1;
				Some(StreamMut::wrap(*(*self.ptr).streams.offset((self.cur - 1) as isize)))
			}
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			((*self.ptr).nb_streams as usize, Some((*self.ptr).nb_streams as usize))
		}
	}
}

impl<'a> ExactSizeIterator for StreamIterMut<'a> { }

pub struct PacketIter<'a> {
	context: &'a mut Context,
}

impl<'a> PacketIter<'a> {
	pub fn new(context: &mut Context) -> PacketIter {
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
