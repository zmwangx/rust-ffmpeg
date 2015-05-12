use std::ptr;

use ffi::*;
use ::format;
use ::Format;

pub struct AudioIter(*mut AVInputFormat);

impl Iterator for AudioIter {
	type Item = Format;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			let ptr = av_input_audio_device_next(self.0);

			if ptr == ptr::null_mut() && self.0 != ptr::null_mut() {
				None
			}
			else {
				self.0 = ptr;

				Some(Format::Input(format::Input::wrap(ptr)))
			}
		}
	}
}

pub fn audio() -> AudioIter {
	AudioIter(ptr::null_mut())
}

pub struct VideoIter(*mut AVInputFormat);

impl Iterator for VideoIter {
	type Item = Format;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			let ptr = av_input_video_device_next(self.0);

			if ptr == ptr::null_mut() && self.0 != ptr::null_mut() {
				None
			}
			else {
				self.0 = ptr;

				Some(Format::Input(format::Input::wrap(ptr)))
			}
		}
	}
}

pub fn video() -> VideoIter {
	VideoIter(ptr::null_mut())
}
