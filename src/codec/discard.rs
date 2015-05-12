use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Discard {
	None,
	Default,
	NonReference,
	Bidirectional,
	NonIntra,
	NonKey,
	All,
}

impl From<AVDiscard> for Discard {
	fn from(value: AVDiscard) -> Self {
		match value {
			AVDISCARD_NONE     => Discard::None,
			AVDISCARD_DEFAULT  => Discard::Default,
			AVDISCARD_NONREF   => Discard::NonReference,
			AVDISCARD_BIDIR    => Discard::Bidirectional,
			AVDISCARD_NONINTRA => Discard::NonIntra,
			AVDISCARD_NONKEY   => Discard::NonKey,
			AVDISCARD_ALL      => Discard::All
		}
	}
}

impl Into<AVDiscard> for Discard {
	fn into(self) -> AVDiscard {
		match self {
			Discard::None          => AVDISCARD_NONE,
			Discard::Default       => AVDISCARD_DEFAULT,
			Discard::NonReference  => AVDISCARD_NONREF,
			Discard::Bidirectional => AVDISCARD_BIDIR,
			Discard::NonIntra      => AVDISCARD_NONINTRA,
			Discard::NonKey        => AVDISCARD_NONKEY,
			Discard::All           => AVDISCARD_ALL
		}
	}
}
