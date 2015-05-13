use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Primaries {
	Reserved0,
	BT709,
	Unspecified,
	Reserved,
	BT470M,

	BT470BG,
	SMPTE170M,
	SMPTE240M,
	Film,
	BT2020,
}

impl From<AVColorPrimaries> for Primaries {
	fn from(value: AVColorPrimaries) -> Primaries {
		match value {
			AVCOL_PRI_RESERVED0   => Primaries::Reserved0,
			AVCOL_PRI_BT709       => Primaries::BT709,
			AVCOL_PRI_UNSPECIFIED => Primaries::Unspecified,
			AVCOL_PRI_RESERVED    => Primaries::Reserved,
			AVCOL_PRI_BT470M      => Primaries::BT470M,

			AVCOL_PRI_BT470BG   => Primaries::BT470BG,
			AVCOL_PRI_SMPTE170M => Primaries::SMPTE170M,
			AVCOL_PRI_SMPTE240M => Primaries::SMPTE240M,
			AVCOL_PRI_FILM      => Primaries::Film,
			AVCOL_PRI_BT2020    => Primaries::BT2020,
			AVCOL_PRI_NB        => Primaries::Reserved0
		}
	}
}

impl Into<AVColorPrimaries> for Primaries {
	fn into(self) -> AVColorPrimaries {
		match self {
			Primaries::Reserved0   => AVCOL_PRI_RESERVED0,
			Primaries::BT709       => AVCOL_PRI_BT709,
			Primaries::Unspecified => AVCOL_PRI_UNSPECIFIED,
			Primaries::Reserved    => AVCOL_PRI_RESERVED,
			Primaries::BT470M      => AVCOL_PRI_BT470M,

			Primaries::BT470BG   => AVCOL_PRI_BT470BG,
			Primaries::SMPTE170M => AVCOL_PRI_SMPTE170M,
			Primaries::SMPTE240M => AVCOL_PRI_SMPTE240M,
			Primaries::Film      => AVCOL_PRI_FILM,
			Primaries::BT2020    => AVCOL_PRI_BT2020,
		}
	}
}
