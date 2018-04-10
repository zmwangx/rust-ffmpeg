use ffi::*;
use libc::c_int;

bitflags! {
    pub struct Flags: c_int {
        const FORCE = SWR_FLAG_RESAMPLE;
    }
}
