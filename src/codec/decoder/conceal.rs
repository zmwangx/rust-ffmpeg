use ffi::*;
use libc::c_int;

bitflags! {
    pub struct Conceal: c_int {
        const GUESS_MVS   = FF_EC_GUESS_MVS;
        const DEBLOCK     = FF_EC_DEBLOCK;
        const FAVOR_INTER = FF_EC_FAVOR_INTER;
    }
}
