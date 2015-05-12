use std::marker::PhantomData;
use std::ptr;

use ffi::*;

pub struct Dictionary<'a> {
	pub ptr: *mut AVDictionary,

	_own:    bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Dictionary<'a> {
	pub fn new() -> Self {
		Dictionary { ptr: ptr::null_mut(), _own: true, _marker: PhantomData }
	}

	pub fn wrap(ptr: *mut AVDictionary) -> Self {
		Dictionary { ptr: ptr, _own: false, _marker: PhantomData }
	}

	pub fn take(&mut self) -> *mut AVDictionary {
		self._own = false;

		self.ptr
	}
}

impl<'a> Drop for Dictionary<'a> {
	fn drop(&mut self) {
		unsafe {
			if self._own && self.ptr != ptr::null_mut() {
				av_dict_free(&mut self.ptr);
			}
		}
	}
}
