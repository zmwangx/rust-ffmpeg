use std::marker::PhantomData;
use std::ptr;

use ffi::*;

pub struct Dictionary<'a> {
	ptr: *mut AVDictionary,

	_own:    bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Dictionary<'a> {
	pub unsafe fn wrap(ptr: *mut AVDictionary) -> Self {
		Dictionary { ptr: ptr, _own: false, _marker: PhantomData }
	}

	pub unsafe fn own(ptr: *mut AVDictionary) -> Self {
		Dictionary { ptr: ptr, _own: true, _marker: PhantomData }
	}

	pub unsafe fn as_ptr(&self) -> *const AVDictionary {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVDictionary {
		self.ptr
	}

	pub unsafe fn take(mut self) -> *mut AVDictionary {
		self._own = false;
		self.ptr
	}
}

impl<'a> Dictionary<'a> {
	pub fn new() -> Self {
		Dictionary { ptr: ptr::null_mut(), _own: true, _marker: PhantomData }
	}
}

impl<'a> Drop for Dictionary<'a> {
	fn drop(&mut self) {
		unsafe {
			if self._own && self.as_ptr() != ptr::null() {
				av_dict_free(&mut self.as_mut_ptr());
			}
		}
	}
}
