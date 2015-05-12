use std::marker::PhantomData;
use std::ptr;

use ffi::*;

pub struct Dictionary<'a> {
	pub ptr: *mut AVDictionary,

	own:     bool,
	_marker: PhantomData<&'a i32>,
}

impl<'a> Dictionary<'a> {
	pub fn new() -> Self {
		Dictionary { ptr: ptr::null_mut(), own: true, _marker: PhantomData }
	}

	pub fn wrap(ptr: *mut AVDictionary) -> Self {
		Dictionary { ptr: ptr, own: false, _marker: PhantomData }
	}

	pub fn take(&mut self) -> *mut AVDictionary {
		self.own = false;
		self.ptr
	}
}

impl<'a> Drop for Dictionary<'a> {
	fn drop(&mut self) {
		unsafe {
			if self.own && self.ptr != ptr::null_mut() {
				av_dict_free(&mut self.ptr);
			}
		}
	}
}
