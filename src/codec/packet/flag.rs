use libc::c_int;
use ffi::*;

bitflags! {
	pub flags Flags: c_int {
		const KEY     = AV_PKT_FLAG_KEY,
		const CORRUPT = AV_PKT_FLAG_CORRUPT,
	}
}
