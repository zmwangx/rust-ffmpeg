use libc::c_int;
use std::mem;
use std::ops::{Deref, DerefMut};

use libc::{int64_t, c_ulonglong};
use ffi::*;
use ::{ChannelLayout, Samples};
use ::util::format;
use super::Frame;

#[derive(PartialEq, Eq)]
pub struct Audio(Frame);

impl Audio {
	pub unsafe fn wrap(ptr: *mut AVFrame) -> Self {
		Audio(Frame::wrap(ptr))
	}
}

impl Audio {
	pub fn empty() -> Self {
		unsafe {
			Audio(Frame::empty())
		}
	}

	pub fn new(format: format::Sample, length: usize, layout: ChannelLayout) -> Self {
		unsafe {
			let mut frame = Audio::empty();

			frame.set_format(format);
			frame.set_length(length);
			frame.set_channel_layout(layout);

			av_frame_get_buffer(frame.as_mut_ptr(), 1);

			frame
		}
	}

	pub fn format(&self) -> format::Sample {
		unsafe {
			if (*self.as_ptr()).format == -1 {
				format::Sample::None
			}
			else {
				format::Sample::from(mem::transmute::<_, AVSampleFormat>(((*self.as_ptr()).format)))
			}
		}
	}

	pub fn set_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.as_mut_ptr()).format = mem::transmute::<AVSampleFormat, c_int>(value.into());
		}
	}

	pub fn channel_layout(&self) -> ChannelLayout {
		unsafe {
			ChannelLayout::from_bits_truncate(av_frame_get_channel_layout(self.as_ptr()) as c_ulonglong)
		}
	}

	pub fn set_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			av_frame_set_channel_layout(self.as_mut_ptr(), value.bits() as int64_t);
		}
	}

	pub fn channels(&self) -> u16 {
		unsafe {
			av_frame_get_channels(self.as_ptr()) as u16
		}
	}

	pub fn set_channels(&mut self, value: u16) {
		unsafe {
			av_frame_set_channels(self.as_mut_ptr(), value as c_int);
		}
	}

	pub fn rate(&self) -> u32 {
		unsafe {
			av_frame_get_sample_rate(self.as_ptr()) as u32
		}
	}

	pub fn set_rate(&mut self, value: u32) {
		unsafe {
			av_frame_set_sample_rate(self.as_mut_ptr(), value as c_int);
		}
	}

	pub fn length(&self) -> usize {
		unsafe {
			(*self.as_ptr()).nb_samples as usize
		}
	}

	pub fn set_length(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).nb_samples = value as c_int;
		}
	}

	pub fn samples(&self) -> Samples {
		unsafe {
			Samples::wrap(self.as_ptr() as *mut AVPicture, self.format(), self.rate(), self.length(), self.channels(), self.channel_layout())
		}
	}

	pub fn samples_mut(&mut self) -> Samples {
		unsafe {
			Samples::wrap(self.as_ptr() as *mut AVPicture, self.format(), self.rate(), self.length(), self.channels(), self.channel_layout())
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
		let mut cloned = Audio::new(self.format(), self.length(), self.channel_layout());
		cloned.clone_from(self);

		cloned
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_frame_copy(self.as_mut_ptr(), source.as_ptr());
			av_frame_copy_props(self.as_mut_ptr(), source.as_ptr());
		}
	}
}
