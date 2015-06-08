use libc::c_int;
use ffi::*;

bitflags! {
	flags Flags: c_int {
		const FORCE = SWR_FLAG_RESAMPLE,
	}
}
