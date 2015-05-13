use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Space {
	RGB,
	BT709,
	Unspecified,
	Reserved,
	FCC,
	BT470BG,
	SMPTE170M,
	SMPTE240M,
	YCOCG,
	YCGCO,
	BT2020NCL,
	BT2020CL,
}

impl Space {
	pub fn name(&self) -> &'static str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr(av_get_colorspace_name((*self).into())).to_bytes())
		}
	}
}

impl From<AVColorSpace> for Space {
	fn from(value: AVColorSpace) -> Self {
		match value {
			AVCOL_SPC_RGB         => Space::RGB,
			AVCOL_SPC_BT709       => Space::BT709,
			AVCOL_SPC_UNSPECIFIED => Space::Unspecified,
			AVCOL_SPC_RESERVED    => Space::Reserved,
			AVCOL_SPC_FCC         => Space::FCC,
			AVCOL_SPC_BT470BG     => Space::BT470BG,
			AVCOL_SPC_SMPTE170M   => Space::SMPTE170M,
			AVCOL_SPC_SMPTE240M   => Space::SMPTE240M,
			AVCOL_SPC_YCOCG       => Space::YCOCG,
			AVCOL_SPC_BT2020_NCL  => Space::BT2020NCL,
			AVCOL_SPC_BT2020_CL   => Space::BT2020CL,
			AVCOL_SPC_NB          => Space::Unspecified
		}
	}
}

impl Into<AVColorSpace> for Space {
	fn into(self) -> AVColorSpace {
		match self {
			Space::RGB         => AVCOL_SPC_RGB,
			Space::BT709       => AVCOL_SPC_BT709,
			Space::Unspecified => AVCOL_SPC_UNSPECIFIED,
			Space::Reserved    => AVCOL_SPC_RESERVED,
			Space::FCC         => AVCOL_SPC_FCC,
			Space::BT470BG     => AVCOL_SPC_BT470BG,
			Space::SMPTE170M   => AVCOL_SPC_SMPTE170M,
			Space::SMPTE240M   => AVCOL_SPC_SMPTE240M,
			Space::YCOCG       => AVCOL_SPC_YCOCG,
			Space::YCGCO       => AVCOL_SPC_YCGCO,
			Space::BT2020NCL   => AVCOL_SPC_BT2020_NCL,
			Space::BT2020CL    => AVCOL_SPC_BT2020_CL
		}
	}
}
