use libc::c_int;
use ffi::*;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Decision {
	Simple,
	Bits,
	RateDistortion,
}

impl From<c_int> for Decision {
	fn from(value: c_int) -> Decision {
		match value {
			FF_MB_DECISION_SIMPLE => Decision::Simple,
			FF_MB_DECISION_BITS   => Decision::Bits,
			FF_MB_DECISION_RD     => Decision::RateDistortion,

			_ => Decision::Simple,
		}
	}
}

impl Into<c_int> for Decision {
	fn into(self) -> c_int {
		match self {
			Decision::Simple         => FF_MB_DECISION_SIMPLE,
			Decision::Bits           => FF_MB_DECISION_BITS,
			Decision::RateDistortion => FF_MB_DECISION_RD,
		}
	}
}
