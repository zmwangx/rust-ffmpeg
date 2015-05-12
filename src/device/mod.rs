pub mod input;
pub mod output;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;
use std::marker::PhantomData;

use ffi::*;

pub struct Info<'a> {
	ptr: *mut AVDeviceInfo,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Info<'a> {
	pub fn wrap(ptr: *mut AVDeviceInfo) -> Self {
		Info { ptr: ptr, _marker: PhantomData }
	}

	pub fn name(&self) -> &'a str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).device_name).to_bytes())
		}
	}

	pub fn description(&self) -> &'a str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).device_description).to_bytes())
		}
	}
}

pub fn register_all() {
	unsafe {
		avdevice_register_all();
	}
}

pub fn version() -> u32 {
	unsafe {
		avdevice_version()
	}
}

pub fn configuration() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avdevice_configuration()).to_bytes())
	}
}

pub fn license() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avdevice_license()).to_bytes())
	}
}
