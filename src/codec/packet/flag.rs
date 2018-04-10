use ffi::*;
use libc::c_int;

bitflags! {
    pub struct Flags: c_int {
        const KEY     = AV_PKT_FLAG_KEY;
        const CORRUPT = AV_PKT_FLAG_CORRUPT;
    }
}
