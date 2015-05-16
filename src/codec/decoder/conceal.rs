use libc::c_int;
use ffi::*;

bitflags! {
	flags Conceal: c_int {
		const CONCEAL_GUESS_MVS   = FF_EC_GUESS_MVS,
		const CONCEAL_DEBLOCK     = FF_EC_DEBLOCK,
		const CONCEAL_FAVOR_INTER = FF_EC_FAVOR_INTER,
	}
}
