use libc::c_int;
use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Compliance {
	VeryStrict,
	Strict,
	Normal,
	Unofficial,
	Experimental,
}

impl From<c_int> for Compliance {
	fn from(value: c_int) -> Self {
		match value {
			FF_COMPLIANCE_VERY_STRICT  => Compliance::VeryStrict,
			FF_COMPLIANCE_STRICT       => Compliance::Strict,
			FF_COMPLIANCE_NORMAL       => Compliance::Normal,
			FF_COMPLIANCE_UNOFFICIAL   => Compliance::Unofficial,
			FF_COMPLIANCE_EXPERIMENTAL => Compliance::Experimental,

			_ => Compliance::Normal
		}
	}
}

impl Into<c_int> for Compliance {
	fn into(self) -> c_int {
		match self {
			Compliance::VeryStrict   => FF_COMPLIANCE_VERY_STRICT,
			Compliance::Strict       => FF_COMPLIANCE_STRICT,
			Compliance::Normal       => FF_COMPLIANCE_NORMAL,
			Compliance::Unofficial   => FF_COMPLIANCE_UNOFFICIAL,
			Compliance::Experimental => FF_COMPLIANCE_EXPERIMENTAL
		}
	}
}
