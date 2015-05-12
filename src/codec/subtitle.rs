use std::mem;

use ffi::*;

pub struct Subtitle {
	pub val: AVSubtitle,
}

impl Subtitle {
	pub fn new() -> Self {
		unsafe {
			Subtitle { val: mem::zeroed() }
		}
	}
}
