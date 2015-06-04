use std::ops::{Deref, DerefMut};

use libc::{c_int, c_float};
use ffi::*;

use super::{Encoder, MotionEstimation, Prediction, Comparison, Decision};
use ::{Packet, Error, Rational};
use ::frame;
use ::format;

pub struct Video(pub Encoder);

impl Video {
	pub fn encode(&mut self, frame: &frame::Video, out: &mut Packet) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			match avcodec_encode_video2(self.as_mut_ptr(), out.as_mut_ptr(), frame.as_ptr(), &mut got) {
				e if e < 0 => Err(Error::from(e)),
				_          => Ok(got != 0)
			}
		}
	}

	pub fn set_width(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).width = value as c_int;
		}
	}

	pub fn set_height(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).height = value as c_int;
		}
	}

	pub fn set_gop(&mut self, value: u32) {
		unsafe {
			(*self.as_mut_ptr()).gop_size = value as c_int;
		}
	}

	pub fn set_format(&mut self, value: format::Pixel) {
		unsafe {
			(*self.as_mut_ptr()).pix_fmt = value.into();
		}
	}

	pub fn set_motion_estimation(&mut self, value: MotionEstimation) {
		unsafe {
			(*self.as_mut_ptr()).me_method = value.into();
		}
	}

	pub fn set_max_b_frames(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).max_b_frames = value as c_int;
		}
	}

	pub fn set_b_quant_factor(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).b_quant_factor = value as c_float;
		}
	}

	pub fn set_b_quant_offset(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).b_quant_offset = value as c_float;
		}
	}

	pub fn set_i_quant_factor(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).i_quant_factor = value as c_float;
		}
	}

	pub fn set_i_quant_offset(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).i_quant_offset = value as c_float;
		}
	}

	pub fn set_lumi_masking(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).lumi_masking = value as c_float;
		}
	}

	pub fn set_temporal_cplx_masking(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).temporal_cplx_masking = value as c_float;
		}
	}

	pub fn set_spatial_cplx_masking(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).spatial_cplx_masking = value as c_float;
		}
	}

	pub fn set_p_masking(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).p_masking = value as c_float;
		}
	}

	pub fn set_dark_masking(&mut self, value: f32) {
		unsafe {
			(*self.as_mut_ptr()).dark_masking = value as c_float;
		}
	}

	pub fn set_prediction(&mut self, value: Prediction) {
		unsafe {
			(*self.as_mut_ptr()).prediction_method = value.into();
		}
	}

	pub fn set_aspect_ratio(&mut self, value: Rational) {
		unsafe {
			(*self.as_mut_ptr()).sample_aspect_ratio = value.0;
		}
	}

	pub fn set_me_comparison(&mut self, value: Comparison) {
		unsafe {
			(*self.as_mut_ptr()).me_cmp = value.into();
		}
	}

	pub fn set_me_sub_comparison(&mut self, value: Comparison) {
		unsafe {
			(*self.as_mut_ptr()).me_sub_cmp = value.into();
		}
	}

	pub fn set_mb_comparison(&mut self, value: Comparison) {
		unsafe {
			(*self.as_mut_ptr()).mb_cmp = value.into();
		}
	}

	pub fn set_ildct_comparison(&mut self, value: Comparison) {
		unsafe {
			(*self.as_mut_ptr()).ildct_cmp = value.into();
		}
	}

	pub fn set_dia_size(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).dia_size = value as c_int;
		}
	}

	pub fn set_last_predictors(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).last_predictor_count = value as c_int;
		}
	}

	pub fn set_pre_me(&mut self, value: MotionEstimation) {
		unsafe {
			(*self.as_mut_ptr()).pre_me = value.into();
		}
	}

	pub fn set_me_pre_comparison(&mut self, value: Comparison) {
		unsafe {
			(*self.as_mut_ptr()).me_pre_cmp = value.into();
		}
	}

	pub fn set_pre_dia_size(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).pre_dia_size = value as c_int;
		}
	}

	pub fn set_me_subpel_quality(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).me_subpel_quality = value as c_int;
		}
	}

	pub fn set_me_range(&mut self, value: usize) {
		unsafe {
			(*self.as_mut_ptr()).me_range = value as c_int;
		}
	}
	
	pub fn set_intra_quant_bias(&mut self, value: Option<usize>) {
		unsafe {
			if let Some(value) = value {
				(*self.as_mut_ptr()).intra_quant_bias = value as c_int;
			}
			else {
				(*self.as_mut_ptr()).intra_quant_bias = FF_DEFAULT_QUANT_BIAS;
			}
		}
	}

	pub fn set_inter_quant_bias(&mut self, value: Option<usize>) {
		unsafe {
			if let Some(value) = value {
				(*self.as_mut_ptr()).inter_quant_bias = value as c_int;
			}
			else {
				(*self.as_mut_ptr()).inter_quant_bias = FF_DEFAULT_QUANT_BIAS;
			}
		}
	}

	pub fn set_mb_decision(&mut self, value: Decision) {
		unsafe {
			(*self.as_mut_ptr()).mb_decision = value.into();
		}
	}
}

impl Deref for Video {
	type Target = Encoder;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Video {
	fn deref_mut(&mut self) -> &mut<Self as Deref>::Target {
		&mut self.0
	}
}
