use std::rc::Rc;
use std::ptr;
use std::mem;

use ffi::*;
use libc::{c_int, c_uint};
use ::{media, Stream, StreamMut, DictionaryRef};
use super::destructor::{self, Destructor};

pub struct Context {
	ptr:  *mut AVFormatContext,
	dtor: Rc<Destructor>,
}

unsafe impl Send for Context { }

impl Context {
	pub unsafe fn wrap(ptr: *mut AVFormatContext, mode: destructor::Mode) -> Self {
		Context { ptr: ptr, dtor: Rc::new(Destructor::new(ptr, mode)) }
	}

	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}

	pub unsafe fn destructor(&self) -> Rc<Destructor> {
		self.dtor.clone()
	}
}

impl Context {
	pub fn stream<'a, 'b>(&'a self, index: usize) -> Option<Stream<'b>> where 'a: 'b {
		unsafe {
			if index >= (*self.as_ptr()).nb_streams as usize {
				None
			}
			else {
				Some(Stream::wrap(self, index))
			}
		}
	}

	pub fn stream_mut<'a, 'b>(&'a mut self, index: usize) -> Option<StreamMut<'b>> where 'a: 'b {
		unsafe {
			if index >= (*self.as_ptr()).nb_streams as usize {
				None
			}
			else {
				Some(StreamMut::wrap(self, index))
			}
		}
	}

	pub fn streams(&self) -> StreamIter {
		StreamIter::new(self)
	}

	pub fn streams_mut(&mut self) -> StreamIterMut {
		StreamIterMut::new(self)
	}

	pub fn duration(&self) -> i64 {
		unsafe {
			(*self.as_ptr()).duration
		}
	}

	pub fn metadata(&self) -> DictionaryRef {
		unsafe {
			DictionaryRef::wrap((*self.as_ptr()).metadata)
		}
	}
}

pub struct Best<'a> {
	context: &'a Context,

	wanted:  i32,
	related: i32,
}

impl<'a> Best<'a> {
	pub unsafe fn new<'b, 'c: 'b>(context: &'c Context) -> Best<'b> {
		Best {
			context: context,

			wanted:  -1,
			related: -1,
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
			let     index   = av_find_best_stream(self.context.as_ptr(),
				kind.into(), self.wanted as c_int, self.related as c_int,
				&mut decoder, 0);

			if index >= 0 && !decoder.is_null() {
				Some(Stream::wrap(self.context, index as usize))
			}
			else {
				None
			}
		}
	}
}

pub struct StreamIter<'a> {
	context: &'a Context,
	current: c_uint,
}

impl<'a> StreamIter<'a> {
	pub fn new<'s, 'c: 's>(context: &'c Context) -> StreamIter<'s> {
		StreamIter { context: context, current: 0 }
	}
}

impl<'a> StreamIter<'a> {
	pub fn wanted<'b, 'c>(&self, stream: &'b Stream) -> Best<'c> where 'a: 'b, 'a: 'c {
		unsafe {
			Best::new(self.context).wanted(stream)
		}
	}

	pub fn related<'b, 'c>(&self, stream: &'b Stream) -> Best<'c> where 'a: 'b, 'a: 'c {
		unsafe {
			Best::new(self.context).related(stream)
		}
	}

	pub fn best<'b>(&self, kind: media::Type) -> Option<Stream<'b>> where 'a: 'b {
		unsafe {
			Best::new(self.context).best(kind)
		}
	}
}

impl<'a> Iterator for StreamIter<'a> {
	type Item = Stream<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.current >= (*self.context.as_ptr()).nb_streams {
				return None;
			}

			self.current += 1;

			Some(Stream::wrap(self.context, (self.current - 1) as usize))
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			let length = (*self.context.as_ptr()).nb_streams as usize;

			(length - self.current as usize, Some(length - self.current as usize))
		}
	}
}

impl<'a> ExactSizeIterator for StreamIter<'a> { }

pub struct StreamIterMut<'a> {
	context: &'a mut Context,
	current: c_uint,
}

impl<'a> StreamIterMut<'a> {
	pub fn new<'s, 'c: 's>(context: &'c mut Context) -> StreamIterMut<'s> {
		StreamIterMut { context: context, current: 0 }
	}
}

impl<'a> Iterator for StreamIterMut<'a> {
	type Item = StreamMut<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.current >= (*self.context.as_ptr()).nb_streams {
				return None
			}

			self.current += 1;

			Some(StreamMut::wrap(mem::transmute_copy(&self.context), (self.current - 1) as usize))
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			let length = (*self.context.as_ptr()).nb_streams as usize;

			(length - self.current as usize, Some(length - self.current as usize))
		}
	}
}

impl<'a> ExactSizeIterator for StreamIterMut<'a> { }
