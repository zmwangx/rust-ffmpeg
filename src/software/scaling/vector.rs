use std::marker::PhantomData;
use std::slice;

use libc::{c_int, c_double};
use ffi::*;

pub struct Vector<'a> {
	pub ptr: *mut SwsVector,

	_own: bool,
	_marker: PhantomData<&'a ()>,
}

impl<'a> Vector<'a> {
	pub fn wrap(ptr: *mut SwsVector) -> Self {
		Vector { ptr: ptr, _own: false, _marker: PhantomData }
	}

	pub fn new(length: usize) -> Self {
		unsafe {
			Vector { ptr: sws_allocVec(length as c_int), _own: true, _marker: PhantomData }
		}
	}

	pub fn gaussian(variance: f64, quality: f64) -> Self {
		unsafe {
			Vector { ptr: sws_getGaussianVec(variance as c_double, quality as c_double), _own: true, _marker: PhantomData }
		}
	}

	pub fn value(value: f64, length: usize) -> Self {
		unsafe {
			Vector { ptr: sws_getConstVec(value as c_double, length as c_int), _own: true, _marker: PhantomData }
		}
	}

	pub fn identity() -> Self {
		unsafe {
			Vector { ptr: sws_getIdentityVec(), _own: true, _marker: PhantomData }
		}
	}

	pub fn scale(&mut self, scalar: f64) {
		unsafe {
			sws_scaleVec(self.ptr, scalar as c_double);
		}
	}

	pub fn normalize(&mut self, height: f64) {
		unsafe {
			sws_normalizeVec(self.ptr, height as c_double);
		}
	}

	pub fn conv(&mut self, other: &Vector) {
		unsafe {
			sws_convVec(self.ptr, other.ptr);
		}
	}

	pub fn add(&mut self, other: &Vector) {
		unsafe {
			sws_addVec(self.ptr, other.ptr);
		}
	}

	pub fn sub(&mut self, other: &Vector) {
		unsafe {
			sws_subVec(self.ptr, other.ptr);
		}
	}

	pub fn shift(&mut self, value: usize) {
		unsafe {
			sws_shiftVec(self.ptr, value as c_int);
		}
	}

	pub fn coefficients(&self) -> &[f64] {
		unsafe {
			slice::from_raw_parts((*self.ptr).coeff, (*self.ptr).length as usize)
		}
	}

	pub fn coefficients_mut(&self) -> &[f64] {
		unsafe {
			slice::from_raw_parts_mut((*self.ptr).coeff, (*self.ptr).length as usize)
		}
	}
}

impl<'a> Clone for Vector<'a> {
	fn clone(&self) -> Self {
		unsafe {
			Vector { ptr: sws_cloneVec(self.ptr), _own: true, _marker: PhantomData }
		}
	}
}

impl<'a> Drop for Vector<'a> {
	fn drop(&mut self) {
		unsafe {
			if self._own {
				sws_freeVec(self.ptr);
			}
		}
	}
}
