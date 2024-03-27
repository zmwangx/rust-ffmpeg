use ffi::*;
use libc::c_int;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct Flags: c_int {
        const FORCED = AV_SUBTITLE_FLAG_FORCED;
    }
}
