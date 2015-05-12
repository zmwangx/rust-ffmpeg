use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum ColorSpace {
	RGB,
	BT709,
	Unspecified,
	Reserved,
	FCC,
	BT470BG,
	SMPTE170M,
	SMPTE240M,
	YCOCG,
	BT2020NCL,
	BT2020CL,
}

impl ColorSpace {
	pub fn name(&self) -> &'static str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr(av_get_colorspace_name((*self).into())).to_bytes())
		}
	}
}

impl From<AVColorSpace> for ColorSpace {
	fn from(value: AVColorSpace) -> Self {
		match value {
			AVCOL_SPC_RGB         => ColorSpace::RGB,
			AVCOL_SPC_BT709       => ColorSpace::BT709,
			AVCOL_SPC_UNSPECIFIED => ColorSpace::Unspecified,
			AVCOL_SPC_RESERVED    => ColorSpace::Reserved,
			AVCOL_SPC_FCC         => ColorSpace::FCC,
			AVCOL_SPC_BT470BG     => ColorSpace::BT470BG,
			AVCOL_SPC_SMPTE170M   => ColorSpace::SMPTE170M,
			AVCOL_SPC_SMPTE240M   => ColorSpace::SMPTE240M,
			AVCOL_SPC_YCOCG       => ColorSpace::YCOCG,
			AVCOL_SPC_BT2020_NCL  => ColorSpace::BT2020NCL,
			AVCOL_SPC_BT2020_CL   => ColorSpace::BT2020CL,
			AVCOL_SPC_NB          => ColorSpace::Unspecified
		}
	}
}

impl Into<AVColorSpace> for ColorSpace {
	fn into(self) -> AVColorSpace {
		match self {
			ColorSpace::RGB         => AVCOL_SPC_RGB,
			ColorSpace::BT709       => AVCOL_SPC_BT709,
			ColorSpace::Unspecified => AVCOL_SPC_UNSPECIFIED,
			ColorSpace::Reserved    => AVCOL_SPC_RESERVED,
			ColorSpace::FCC         => AVCOL_SPC_FCC,
			ColorSpace::BT470BG     => AVCOL_SPC_BT470BG,
			ColorSpace::SMPTE170M   => AVCOL_SPC_SMPTE170M,
			ColorSpace::SMPTE240M   => AVCOL_SPC_SMPTE240M,
			ColorSpace::YCOCG       => AVCOL_SPC_YCOCG,
			ColorSpace::BT2020NCL   => AVCOL_SPC_BT2020_NCL,
			ColorSpace::BT2020CL    => AVCOL_SPC_BT2020_CL
		}
	}
}
