use libc::c_int;
use std::mem;
use std::ops::{Deref, DerefMut};

use ffi::*;
use ::util::format;
use super::Frame;

pub struct Audio(Frame);

impl Audio {
	pub fn empty() -> Self {
		unsafe {
			Audio(Frame { ptr: av_frame_alloc() })
		}
	}

	pub fn new(format: format::Sample, samples: usize, layout: i64) -> Self {
		unsafe {
			let mut frame = Audio::empty();

			frame.set_format(format);
			frame.set_samples(samples);
			frame.set_channel_layout(layout);

			av_frame_get_buffer(frame.ptr, 1);

			frame
		}
	}

	pub fn format(&self) -> format::Sample {
		unsafe {
			if (*self.ptr).format == -1 {
				format::Sample::None
			}
			else {
				format::Sample::from(mem::transmute::<_, AVSampleFormat>(((*self.ptr).format)))
			}
		}
	}

	pub fn set_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.ptr).format = mem::transmute::<AVSampleFormat, c_int>(value.into());
		}
	}

	pub fn channel_layout(&self) -> i64 {
		unsafe {
			av_frame_get_channel_layout(self.ptr)
		}
	}

	pub fn set_channel_layout(&mut self, value: i64) {
		unsafe {
			av_frame_set_channel_layout(self.ptr, value);
		}
	}

	pub fn channels(&self) -> usize {
		unsafe {
			av_frame_get_channels(self.ptr) as usize
		}
	}

	pub fn set_channels(&mut self, value: usize) {
		unsafe {
			av_frame_set_channels(self.ptr, value as c_int);
		}
	}

	pub fn rate(&self) -> i32 {
		unsafe {
			av_frame_get_sample_rate(self.ptr)
		}
	}

	pub fn set_rate(&mut self, value: i32) {
		unsafe {
			av_frame_set_sample_rate(self.ptr, value);
		}
	}

	pub fn samples(&self) -> usize {
		unsafe {
			(*self.ptr).nb_samples as usize
		}
	}

	pub fn set_samples(&mut self, value: usize) {
		unsafe {
			(*self.ptr).nb_samples = value as c_int;
		}
	}
}

unsafe impl Send for Audio { }

impl Deref for Audio {
	type Target = Frame;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Audio {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}

impl Clone for Audio {
	fn clone(&self) -> Self {
		let mut cloned = Audio::new(self.format(), self.samples(), self.channel_layout());
		cloned.clone_from(self);

		cloned
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.ptr, source.ptr);
			av_frame_copy_props(self.ptr, source.ptr);
		}
	}
}
