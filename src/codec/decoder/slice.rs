use libc::c_int;
use ffi::*;

bitflags! {
	flags Flags: c_int {
		const CODED_ORDER = SLICE_FLAG_CODED_ORDER,
		const ALLOW_FIELD = SLICE_FLAG_ALLOW_FIELD,
		const ALLOW_PLANE = SLICE_FLAG_ALLOW_PLANE,
	}
}
