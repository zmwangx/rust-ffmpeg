use libc::c_int;
use ffi::*;

bitflags! {
	flags Flags: c_int {
		const FORCED = AV_SUBTITLE_FLAG_FORCED,
	}
}
