use std::marker::PhantomData;
use std::slice;

use ffi::*;
use super::Packet;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Type {
	Palette,
	NewExtraData,
	ParamChange,
	H263MbInfo,
	ReplayGain,
	DisplayMatrix,
	Stereo3d,
	AudioServiceType,
	SkipSamples,
	JpDualMono,
	StringsMetadata,
	SubtitlePosition,
	MatroskaBlockAdditional,
	WebVTTIdentifier,
	WebVTTSettings,
	MetadataUpdate,
}

impl From<AVPacketSideDataType> for Type {
	fn from(value: AVPacketSideDataType) -> Self {
		match value {
			AV_PKT_DATA_PALETTE                  => Type::Palette,
			AV_PKT_DATA_NEW_EXTRADATA            => Type::NewExtraData,
			AV_PKT_DATA_PARAM_CHANGE             => Type::ParamChange,
			AV_PKT_DATA_H263_MB_INFO             => Type::H263MbInfo,
			AV_PKT_DATA_REPLAYGAIN               => Type::ReplayGain,
			AV_PKT_DATA_DISPLAYMATRIX            => Type::DisplayMatrix,
			AV_PKT_DATA_STEREO3D                 => Type::Stereo3d,
			AV_PKT_DATA_AUDIO_SERVICE_TYPE       => Type::AudioServiceType,
			AV_PKT_DATA_SKIP_SAMPLES             => Type::SkipSamples,
			AV_PKT_DATA_JP_DUALMONO              => Type::JpDualMono,
			AV_PKT_DATA_STRINGS_METADATA         => Type::StringsMetadata,
			AV_PKT_DATA_SUBTITLE_POSITION        => Type::SubtitlePosition,
			AV_PKT_DATA_MATROSKA_BLOCKADDITIONAL => Type::MatroskaBlockAdditional,
			AV_PKT_DATA_WEBVTT_IDENTIFIER        => Type::WebVTTIdentifier,
			AV_PKT_DATA_WEBVTT_SETTINGS          => Type::WebVTTSettings,
			AV_PKT_DATA_METADATA_UPDATE          => Type::MetadataUpdate
		}
	}
}

impl Into<AVPacketSideDataType> for Type {
	fn into(self) -> AVPacketSideDataType {
		match self {
			Type::Palette                 => AV_PKT_DATA_PALETTE,
			Type::NewExtraData            => AV_PKT_DATA_NEW_EXTRADATA,
			Type::ParamChange             => AV_PKT_DATA_PARAM_CHANGE,
			Type::H263MbInfo              => AV_PKT_DATA_H263_MB_INFO,
			Type::ReplayGain              => AV_PKT_DATA_REPLAYGAIN,
			Type::DisplayMatrix           => AV_PKT_DATA_DISPLAYMATRIX,
			Type::Stereo3d                => AV_PKT_DATA_STEREO3D,
			Type::AudioServiceType        => AV_PKT_DATA_AUDIO_SERVICE_TYPE,
			Type::SkipSamples             => AV_PKT_DATA_SKIP_SAMPLES,
			Type::JpDualMono              => AV_PKT_DATA_JP_DUALMONO,
			Type::StringsMetadata         => AV_PKT_DATA_STRINGS_METADATA,
			Type::SubtitlePosition        => AV_PKT_DATA_SUBTITLE_POSITION,
			Type::MatroskaBlockAdditional => AV_PKT_DATA_MATROSKA_BLOCKADDITIONAL,
			Type::WebVTTIdentifier        => AV_PKT_DATA_WEBVTT_IDENTIFIER,
			Type::WebVTTSettings          => AV_PKT_DATA_WEBVTT_SETTINGS,
			Type::MetadataUpdate          => AV_PKT_DATA_METADATA_UPDATE
		}
	}
}

pub struct SideData<'a> {
	ptr: *mut AVPacketSideData,

	_marker: PhantomData<&'a Packet>
}

impl<'a> SideData<'a> {
	pub fn wrap(ptr: *mut AVPacketSideData) -> Self {
		SideData { ptr: ptr, _marker: PhantomData }
	}

	pub fn kind(&self) -> Type {
		unsafe {
			Type::from((*self.ptr).kind)
		}
	}

	pub fn data(&'a self) -> &'a [u8] {
		unsafe {
			slice::from_raw_parts((*self.ptr).data, (*self.ptr).size as usize)
		}
	}
}
