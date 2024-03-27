use ffi::*;
use libc::c_int;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct Flags: c_int {
        const CORRUPT = AV_FRAME_FLAG_CORRUPT;
    }
}
