use libc::c_int;
use ffi::*;

bitflags! {
	flags Flags: c_int {
		const CORRUPT = AV_FRAME_FLAG_CORRUPT,
	}
}
