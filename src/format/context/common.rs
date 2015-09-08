use std::marker::PhantomData;
use std::ptr;

use ffi::*;
use libc::{c_int, c_uint};
use ::{media, Stream, StreamMut, DictionaryRef};

pub struct Context {
	ptr: *mut AVFormatContext,
}

unsafe impl Send for Context { }

impl Context {
	pub unsafe fn wrap(ptr: *mut AVFormatContext) -> Self {
		Context { ptr: ptr }
	}

	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}
}

impl Context {
	pub fn stream<'a, 'b>(&'a self, index: usize) -> Option<Stream<'b>> where 'a: 'b {
		unsafe {
			if index >= (*self.as_ptr()).nb_streams as usize {
				None
			}
			else {
				Some(Stream::wrap(*(*self.as_ptr()).streams.offset(index as isize)))
			}
		}
	}

	pub fn stream_mut<'a, 'b>(&'a mut self, index: usize) -> Option<StreamMut<'b>> where 'a: 'b {
		unsafe {
			if index >= (*self.as_ptr()).nb_streams as usize {
				None
			}
			else {
				Some(StreamMut::wrap(*(*self.as_mut_ptr()).streams.offset(index as isize)))
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

	pub fn metadata(&self) -> DictionaryRef {
		unsafe {
			DictionaryRef::wrap((*self.as_ptr()).metadata)
		}
	}
}

pub struct Best<'a> {
	ptr: *const AVFormatContext,

	wanted:  i32,
	related: i32,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Best<'a> {
	pub unsafe fn new<'b>(ptr: *const AVFormatContext) -> Best<'b> {
		Best {
			ptr: ptr,

			wanted:  -1,
			related: -1,

			_marker: PhantomData,
		}
	}

	pub fn wanted<'b>(mut self, stream: &'b Stream) -> Best<'a> where 'a: 'b {
		self.wanted = stream.index() as i32;
		self
	}

	pub fn related<'b>(mut self, stream: &'b Stream) -> Best<'a> where 'a: 'b {
		self.related = stream.index() as i32;
		self
	}

	pub fn best<'b>(self, kind: media::Type) -> Option<Stream<'b>> where 'a: 'b {
		unsafe {
			let mut decoder = ptr::null_mut();
			let     index   = av_find_best_stream(self.ptr,
				kind.into(), self.wanted as c_int, self.related as c_int,
				&mut decoder, 0);

			if index >= 0 && !decoder.is_null() {
				Some(Stream::wrap(*(*self.ptr).streams.offset(index as isize)))
			}
			else {
				None
			}
		}
	}
}

pub struct StreamIter<'a> {
	ptr: *const AVFormatContext,
	cur: c_uint,

	_marker: PhantomData<&'a ()>,
}

impl<'a> StreamIter<'a> {
	pub fn new(ptr: *const AVFormatContext) -> Self {
		StreamIter { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> StreamIter<'a> {
	pub fn wanted<'b: 'a, 'c: 'a>(&'a self, stream: &'b Stream) -> Best<'a> {
		unsafe {
			Best::new(self.ptr).wanted(stream)
		}
	}

	pub fn related<'b: 'a>(&'a self, stream: &'b Stream) -> Best<'a> {
		unsafe {
			Best::new(self.ptr).related(stream)
		}
	}

	pub fn best<'b: 'a>(&'a self, kind: media::Type) -> Option<Stream<'b>> {
		unsafe {
			Best::new(self.ptr).best(kind)
		}
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

	_marker: PhantomData<&'a ()>,
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
