use ffi::*;
use super::Vector;

pub struct Filter {
	pub ptr: *mut SwsFilter,
}

impl Filter {
	pub fn get(luma_g_blur: f32, chroma_g_blur: f32,
	           luma_sharpen: f32, chroma_sharpen: f32,
	           chroma_h_shift: f32, chroma_v_shift: f32) -> Self {
		unsafe {
			Filter {
				ptr: sws_getDefaultFilter(luma_g_blur, chroma_g_blur,
				                          luma_sharpen, chroma_sharpen,
				                          chroma_h_shift, chroma_v_shift, 0)
			}
		}
	}

	pub fn new() -> Self {
		Self::get(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
	}

	pub fn luma_horizontal(&self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumH)
		}
	}
	
	pub fn luma_horizontal_mut(&mut self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumH)
		}
	}

	pub fn luma_vertical(&self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumV)
		}
	}
	
	pub fn luma_vertical_mut(&mut self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumV)
		}
	}

	pub fn chroma_horizontal(&self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumV)
		}
	}
	
	pub fn chroma_horizontal_mut(&mut self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumV)
		}
	}

	pub fn chroma_vertical(&self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumV)
		}
	}
	
	pub fn chroma_vertical_mut(&mut self) -> Vector {
		unsafe {
			Vector::wrap((*self.ptr).lumV)
		}
	}
}

impl Drop for Filter {
	fn drop(&mut self) {
		unsafe {
			sws_freeFilter(self.ptr);
		}
	}
}
