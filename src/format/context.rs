use std::ffi::CString;
use std::ptr;
use std::path::Path;
use std::marker::PhantomData;

use libc::c_uint;
use ffi::*;
use ::{Error, Dictionary, Codec, Stream, Format, Packet};

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
}

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

		match packet.read(self.context) {
			Ok(..) => unsafe {
				let stream = Stream::wrap(*(*self.context.as_ptr()).streams.offset(packet.stream() as isize));

				Some((stream, packet))
			},

			_ =>
				None
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
