use std::marker::PhantomData;
use std::slice;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;
use super::Frame;
use ::DictionaryRef;

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
	#[inline]
	pub fn name(&self) -> &'static str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr(av_frame_side_data_name((*self).into())).to_bytes())
		}
	}
}

impl From<AVFrameSideDataType> for Type {
	#[inline(always)]
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
	#[inline(always)]
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
	ptr: *mut AVFrameSideData,

	_marker: PhantomData<&'a Frame>,
}

impl<'a> SideData<'a> {
	#[inline(always)]
	pub unsafe fn wrap(ptr: *mut AVFrameSideData) -> Self {
		SideData { ptr: ptr, _marker: PhantomData }
	}

	#[inline(always)]
	pub unsafe fn as_ptr(&self) -> *const AVFrameSideData {
		self.ptr as *const _
	}

	#[inline(always)]
	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFrameSideData {
		self.ptr
	}
}

impl<'a> SideData<'a> {
	#[inline]
	pub fn kind(&self) -> Type {
		unsafe {
			Type::from((*self.as_ptr()).kind)
		}
	}

	#[inline]
	pub fn data(&self) -> &[u8] {
		unsafe {
			slice::from_raw_parts((*self.as_ptr()).data, (*self.as_ptr()).size as usize)
		}
	}

	#[inline]
	pub fn metadata(&self) -> DictionaryRef {
		unsafe {
			DictionaryRef::wrap((*self.as_ptr()).metadata)
		}
	}
}
