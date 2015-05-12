use std::ffi::CString;
use std::ptr;
use std::path::Path;
use std::marker::PhantomData;
use std::ops::Deref;

use libc::{c_int, c_uint};
use ffi::*;
use ::{Error, Dictionary, Codec, Stream, Format};
use ::device;

pub struct Context<'a> {
	pub ptr: *mut AVFormatContext,

	_input:  bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Context<'a> {
	pub fn new() -> Self {
		unsafe {
			Context {
				ptr: avformat_alloc_context(),

				_input:  false,
				_marker: PhantomData,
			}
		}
	}

	pub fn input(ptr: *mut AVFormatContext) -> Self {
		Context {
			ptr: ptr,

			_input:  true,
			_marker: PhantomData,
		}
	}

	pub fn streams(&'a self) -> StreamIter<'a> {
		StreamIter::new(self.ptr)
	}

	pub fn devices(&'a self) -> Result<DeviceIter<'a>, Error> {
		DeviceIter::new(self.ptr)
	}

	pub fn probe_score(&self) -> i32 {
		unsafe {
			av_format_get_probe_score(self.ptr)
		}
	}

	pub fn video_codec(&'a self) -> Option<Codec<'a>> {
		unsafe {
			let ptr = av_format_get_video_codec(self.ptr);

			if ptr == ptr::null_mut() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_video_codec(&'a mut self, value: Codec<'a>) {
		unsafe {
			av_format_set_video_codec(self.ptr, value.ptr);
		}
	}

	pub fn audio_codec(&'a self) -> Option<Codec<'a>> {
		unsafe {
			let ptr = av_format_get_audio_codec(self.ptr);

			if ptr == ptr::null_mut() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_audio_codec(&'a mut self, value: Codec<'a>) {
		unsafe {
			av_format_set_audio_codec(self.ptr, value.ptr);
		}
	}

	pub fn subtitle_codec(&'a self) -> Option<Codec<'a>> {
		unsafe {
			let ptr = av_format_get_subtitle_codec(self.ptr);

			if ptr == ptr::null_mut() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_subtitle_codec(&'a mut self, value: Codec<'a>) {
		unsafe {
			av_format_set_subtitle_codec(self.ptr, value.ptr);
		}
	}

	pub fn data_codec(&'a self) -> Option<Codec<'a>> {
		unsafe {
			let ptr = av_format_get_data_codec(self.ptr);

			if ptr == ptr::null_mut() {
				None
			}
			else {
				Some(Codec::wrap(ptr))
			}
		}
	}

	pub fn set_data_codec(&'a mut self, value: Codec<'a>) {
		unsafe {
			av_format_set_data_codec(self.ptr, value.ptr);
		}
	}

	pub fn packet(&'a self) -> Packet<'a> {
		Packet::new(self.ptr)
	}
}

impl<'a> Drop for Context<'a> {
	fn drop(&mut self) {
		unsafe {
			if self._input {
				avformat_close_input(&mut self.ptr);
			}
			else {
				avformat_free_context(self.ptr);
			}
		}
	}
}

pub struct Packet<'a> {
	ptr: *mut AVFormatContext,
	pkt: ::Packet,

	_marker: PhantomData<&'a Context<'a>>,
}

impl<'a> Packet<'a> {
	pub fn new(ptr: *mut AVFormatContext) -> Self {
		Packet { ptr: ptr, pkt: ::Packet::new(), _marker: PhantomData }
	}

	pub fn stream(&'a self) -> Stream<'a> {
		unsafe {
			Stream::wrap(*(*self.ptr).streams.offset(self.pkt.val.stream_index as isize))
		}
	}

	pub fn read(&mut self) -> Result<(), Error> {
		unsafe {
			match av_read_frame(self.ptr, &mut self.pkt.val) {
				0 => Ok(()),
				e => Err(Error::new(e))
			}
		}
	}

	pub fn write(&mut self) -> Result<bool, Error> {
		unsafe {
			match av_write_frame(self.ptr, &mut self.pkt.val) {
				1 => Ok(true),
				0 => Ok(false),
				e => Err(Error::new(e))
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

pub struct StreamIter<'a> {
	ptr: *const AVFormatContext,
	cur: c_uint,

	_marker: PhantomData<&'a Context<'a>>,
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

pub struct DeviceIter<'a> {
	ptr: *mut AVDeviceInfoList,
	cur: c_int,

	_marker: PhantomData<&'a ()>,
}

impl<'a> DeviceIter<'a> {
	pub fn new(ctx: *mut AVFormatContext) -> Result<Self, Error> {
		unsafe {
			let mut ptr: *mut AVDeviceInfoList = ptr::null_mut();

			match avdevice_list_devices(ctx, &mut ptr) {
				n if n < 0 =>
					Err(Error::new(n)),

				_ =>
					Ok(DeviceIter { ptr: ptr, cur: 0, _marker: PhantomData })
			}
		}
	}

	pub fn default(&self) -> usize {
		unsafe {
			(*self.ptr).default_device as usize
		}
	}
}

impl<'a> Drop for DeviceIter<'a> {
	fn drop(&mut self) {
		unsafe {
			avdevice_free_list_devices(&mut self.ptr);
		}
	}
}

impl<'a> Iterator for DeviceIter<'a> {
	type Item = device::Info<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).nb_devices {
				None
			}
			else {
				self.cur += 1;
				Some(device::Info::wrap(*(*self.ptr).devices.offset((self.cur - 1) as isize)))
			}
		}
	}
}

pub fn open<'a>(path: &Path) -> Result<Context<'a>, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
		let     status = avformat_open_input(&mut ps, path, ptr::null_mut(), ptr::null_mut());

		match status {
			0 => {
				let ctx = Context::input(ps);

				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(ctx),
					e => Err(Error::new(e))
				}
			},

			e => Err(Error::new(e))
		}
	}
}

pub fn open_with<'a>(path: &Path, mut options: Dictionary) -> Result<Context<'a>, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
		let     opts   = &mut options.ptr;
		let     status = avformat_open_input(&mut ps, path, ptr::null_mut(), opts);

		av_dict_free(opts);

		match status {
			0 => {
				let ctx = Context::input(ps);

				match avformat_find_stream_info(ps, ptr::null_mut()) {
					0 => Ok(ctx),
					e => Err(Error::new(e))
				}
			},

			e => Err(Error::new(e))
		}
	}
}

pub fn open_as<'a>(path: &Path, format: &Format) -> Result<Context<'a>, Error> {
	if let &Format::Input(ref format) = format {
		unsafe {
			let mut ps     = ptr::null_mut();
			let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
			let     status = avformat_open_input(&mut ps, path, format.ptr, ptr::null_mut());

			match status {
				0 => {
					let ctx = Context::input(ps);

					match avformat_find_stream_info(ps, ptr::null_mut()) {
						0 => Ok(ctx),
						e => Err(Error::new(e))
					}
				},

				e => Err(Error::new(e))
			}
		}
	}
	else {
		Err(Error::new(AVERROR_BUG))
	}
}

pub fn open_as_with<'a>(path: &Path, format: &Format, mut options: Dictionary) -> Result<Context<'a>, Error> {
	if let &Format::Input(ref format) = format {
		unsafe {
			let mut ps     = ptr::null_mut();
			let     path   = path.as_os_str().to_cstring().unwrap().as_ptr();
			let     opts   = &mut options.ptr;
			let     status = avformat_open_input(&mut ps, path, format.ptr, opts);

			av_dict_free(opts);

			match status {
				0 => {
					let ctx = Context::input(ps);

					match avformat_find_stream_info(ps, ptr::null_mut()) {
						0 => Ok(ctx),
						e => Err(Error::new(e))
					}
				},

				e => Err(Error::new(e))
			}
		}
	}
	else {
		Err(Error::new(AVERROR_BUG))
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
