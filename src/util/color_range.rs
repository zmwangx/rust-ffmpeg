use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum ColorRange {
	Unspecified,
	MPEG,
	JPEG,
}

impl From<AVColorRange> for ColorRange {
	fn from(value: AVColorRange) -> Self {
		match value {
			AVCOL_RANGE_UNSPECIFIED => ColorRange::Unspecified,
			AVCOL_RANGE_MPEG        => ColorRange::MPEG,
			AVCOL_RANGE_JPEG        => ColorRange::JPEG,
			AVCOL_RANGE_NB          => ColorRange::Unspecified
		}
	}
}

impl Into<AVColorRange> for ColorRange {
	fn into(self) -> AVColorRange {
		match self {
			ColorRange::Unspecified => AVCOL_RANGE_UNSPECIFIED,
			ColorRange::MPEG        => AVCOL_RANGE_MPEG,
			ColorRange::JPEG        => AVCOL_RANGE_JPEG
		}
	}
}
