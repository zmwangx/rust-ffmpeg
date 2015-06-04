use std::ffi::CString;
use std::ptr;
use std::path::Path;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use libc::c_uint;
use ffi::*;
use ::{Error, Dictionary, Codec, Stream, Format};

pub struct Context {
	ptr: *mut AVFormatContext,

	_input: bool,
}

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

	pub fn streams(&self) -> StreamIter {
		unsafe {
			StreamIter::new(self.as_ptr())
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

			if ptr == ptr::null_mut() {
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

			if ptr == ptr::null_mut() {
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

			if ptr == ptr::null_mut() {
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

			if ptr == ptr::null_mut() {
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

	pub fn packet(&mut self) -> Packet {
		unsafe {
			Packet::new(self.as_mut_ptr())
		}
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

pub struct Packet<'a> {
	ptr: *mut AVFormatContext,
	pkt: ::Packet,

	_marker: PhantomData<&'a Context>,
}

impl<'a> Packet<'a> {
	pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
		self.ptr
	}
}

impl<'a> Packet<'a> {
	pub fn new(ptr: *mut AVFormatContext) -> Self {
		Packet { ptr: ptr, pkt: ::Packet::empty(), _marker: PhantomData }
	}

	pub fn stream(&self) -> Stream {
		unsafe {
			Stream::wrap(*(*self.as_ptr()).streams.offset((*self.pkt.as_ptr()).stream_index as isize))
		}
	}

	pub fn read(&mut self) -> Result<(), Error> {
		unsafe {
			match av_read_frame(self.as_mut_ptr(), self.pkt.as_mut_ptr()) {
				0 => Ok(()),
				e => Err(Error::from(e))
			}
		}
	}

	pub fn write(&mut self) -> Result<bool, Error> {
		unsafe {
			match av_write_frame(self.as_mut_ptr(), self.pkt.as_mut_ptr()) {
				1 => Ok(true),
				0 => Ok(false),
				e => Err(Error::from(e))
			}
		}
	}
}

impl<'a> Deref for Packet<'a> {
	type Target = ::Packet;

	fn deref<'b>(&'b self) -> &'b ::Packet {
		&self.pkt
	}
}

impl<'a> DerefMut for Packet<'a> {
	fn deref_mut<'b>(&'b mut self) -> &'b mut ::Packet {
		&mut self.pkt
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
}

pub fn open(path: &Path) -> Result<Context, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
		let     status = avformat_open_input(&mut ps, path, ptr::null_mut(), ptr::null_mut());

		match status {
			0 => {
				let ctx = Context::input(ps);

				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(ctx),
					e => Err(Error::from(e))
				}
			},

			e => Err(Error::from(e))
		}
	}
}

pub fn open_with(path: &Path, options: Dictionary) -> Result<Context, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
		let mut opts   = options.take();
		let     status = avformat_open_input(&mut ps, path, ptr::null_mut(), &mut opts);

		Dictionary::own(opts);

		match status {
			0 => {
				let ctx = Context::input(ps);

				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(ctx),
					e => Err(Error::from(e))
				}
			},

			e => Err(Error::from(e))
		}
	}
}

pub fn open_as(path: &Path, format: &Format) -> Result<Context, Error> {
	if let &Format::Input(ref format) = format {
		unsafe {
			let mut ps     = ptr::null_mut();
			let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
			let     status = avformat_open_input(&mut ps, path, format.as_ptr(), ptr::null_mut());

			match status {
				0 => {
					let ctx = Context::input(ps);

					match avformat_find_stream_info(ps, ptr::null_mut()) {
						0 => Ok(ctx),
						e => Err(Error::from(e))
					}
				},

				e => Err(Error::from(e))
			}
		}
	}
	else {
		Err(Error::Bug)
	}
}

pub fn open_as_with(path: &Path, format: &Format, options: Dictionary) -> Result<Context, Error> {
	if let &Format::Input(ref format) = format {
		unsafe {
			let mut ps     = ptr::null_mut();
			let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
			let mut opts   = options.take();
			let     status = avformat_open_input(&mut ps, path, format.as_ptr(), &mut opts);

			Dictionary::own(opts);

			match status {
				0 => {
					let ctx = Context::input(ps);

					match avformat_find_stream_info(ps, ptr::null_mut()) {
						0 => Ok(ctx),
						e => Err(Error::from(e))
					}
				},

				e => Err(Error::from(e))
			}
		}
	}
	else {
		Err(Error::Bug)
	}
}

pub fn dump(ctx: &Context, index: i32, url: Option<&str>) {
	let url = if let Some(url) = url {
		CString::new(url).unwrap().as_ptr()
	}
	else {
		ptr::null()
	};

	unsafe {
		if ctx._input {
			av_dump_format(ctx.ptr, index, url, 0);
		}
		else {
			av_dump_format(ctx.ptr, index, url, 1);
		}
	}
}
