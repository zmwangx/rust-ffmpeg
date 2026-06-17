use ffi::*;
use libc::c_int;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct DecodeError: c_int {
        const INVALID_BITSTREAM = FF_DECODE_ERROR_INVALID_BITSTREAM;
        const MISSING_REFERENCE = FF_DECODE_ERROR_MISSING_REFERENCE;
        #[cfg(feature = "ffmpeg_5_1")]
        const CONCEALMENT_ACTIVE = FF_DECODE_ERROR_CONCEALMENT_ACTIVE;
        #[cfg(feature = "ffmpeg_5_1")]
        const DECODE_SLICES = FF_DECODE_ERROR_DECODE_SLICES;
    }
}
