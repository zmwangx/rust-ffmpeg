use std::marker::PhantomData;
use std::slice;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;
use super::Frame;
use ::Dictionary;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Type {
	PanScan,
	A53CC,
	Stereo3D,
	MatrixEncoding,
	DownMixInfo,
	ReplayGain,
	DisplayMatrix,
	AFD,
	MotionVectors,
	SkipSamples,
	AudioServiceType,
}

impl Type {
	pub fn name(&self) -> &'static str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr(av_frame_side_data_name((*self).into())).to_bytes())
		}
	}
}

impl From<AVFrameSideDataType> for Type {
	fn from(value: AVFrameSideDataType) -> Self {
		match value {
			AV_FRAME_DATA_PANSCAN            => Type::PanScan,
			AV_FRAME_DATA_A53_CC             => Type::A53CC,
			AV_FRAME_DATA_STEREO3D           => Type::Stereo3D,
			AV_FRAME_DATA_MATRIXENCODING     => Type::MatrixEncoding,
			AV_FRAME_DATA_DOWNMIX_INFO       => Type::DownMixInfo,
			AV_FRAME_DATA_REPLAYGAIN         => Type::ReplayGain,
			AV_FRAME_DATA_DISPLAYMATRIX      => Type::DisplayMatrix,
			AV_FRAME_DATA_AFD                => Type::AFD,
			AV_FRAME_DATA_MOTION_VECTORS     => Type::MotionVectors,
			AV_FRAME_DATA_SKIP_SAMPLES       => Type::SkipSamples,
			AV_FRAME_DATA_AUDIO_SERVICE_TYPE => Type::AudioServiceType
		}
	}
}

impl Into<AVFrameSideDataType> for Type {
	fn into(self) -> AVFrameSideDataType {
		match self {
			Type::PanScan          => AV_FRAME_DATA_PANSCAN,
			Type::A53CC            => AV_FRAME_DATA_A53_CC,
			Type::Stereo3D         => AV_FRAME_DATA_STEREO3D,
			Type::MatrixEncoding   => AV_FRAME_DATA_MATRIXENCODING,
			Type::DownMixInfo      => AV_FRAME_DATA_DOWNMIX_INFO,
			Type::ReplayGain       => AV_FRAME_DATA_REPLAYGAIN,
			Type::DisplayMatrix    => AV_FRAME_DATA_DISPLAYMATRIX,
			Type::AFD              => AV_FRAME_DATA_AFD,
			Type::MotionVectors    => AV_FRAME_DATA_MOTION_VECTORS,
			Type::SkipSamples      => AV_FRAME_DATA_SKIP_SAMPLES,
			Type::AudioServiceType => AV_FRAME_DATA_AUDIO_SERVICE_TYPE
		}
	}
}

pub struct SideData<'a> {
	pub ptr: *mut AVFrameSideData,

	_marker: PhantomData<&'a Frame>,
}

impl<'a> SideData<'a> {
	pub fn wrap(ptr: *mut AVFrameSideData) -> Self {
		SideData { ptr: ptr, _marker: PhantomData }
	}

	pub fn kind(&self) -> Type {
		unsafe {
			Type::from((*self.ptr).kind)
		}
	}

	pub fn data(&self) -> &[u8] {
		unsafe {
			slice::from_raw_parts((*self.ptr).data, (*self.ptr).size as usize)
		}
	}

	pub fn metadata(&self) -> Dictionary {
		unsafe {
			Dictionary::wrap((*self.ptr).metadata)
		}
	}
}
