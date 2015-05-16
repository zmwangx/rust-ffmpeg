use libc::c_int;
use ffi::*;

bitflags! {
	flags Check: c_int {
		const CHECK_CRC      = AV_EF_CRCCHECK,
		const CHECK_BISTREAM = AV_EF_BITSTREAM,
		const CHECK_BUFFER   = AV_EF_BUFFER,
		const CHECK_EXPLODE  = AV_EF_EXPLODE,

		const CHECK_IGNORE_ERROR = AV_EF_IGNORE_ERR,
		const CHECK_CAREFUL      = AV_EF_CAREFUL,
		const CHECK_COMPLIANT    = AV_EF_COMPLIANT,
		const CHECK_AGGRESSIVE   = AV_EF_AGGRESSIVE,
	}
}
