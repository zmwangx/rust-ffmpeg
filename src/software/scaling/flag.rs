use crate::ffi::*;
#[cfg(feature = "ffmpeg_8_0")]
use crate::software::scaling::SwsFlags::*;
use libc::c_int;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Flags: c_int {
        const FAST_BILINEAR        = SWS_FAST_BILINEAR as _;
        const BILINEAR             = SWS_BILINEAR as _;
        const BICUBIC              = SWS_BICUBIC as _;
        const X                    = SWS_X as _;
        const POINT                = SWS_POINT as _;
        const AREA                 = SWS_AREA as _;
        const BICUBLIN             = SWS_BICUBLIN as _;
        const GAUSS                = SWS_GAUSS as _;
        const SINC                 = SWS_SINC as _;
        const LANCZOS              = SWS_LANCZOS as _;
        const SPLINE               = SWS_SPLINE as _;
        const SRC_V_CHR_DROP_MASK  = SWS_SRC_V_CHR_DROP_MASK as _;
        const SRC_V_CHR_DROP_SHIFT = SWS_SRC_V_CHR_DROP_SHIFT as _;
        const PARAM_DEFAULT        = SWS_PARAM_DEFAULT as _;
        const PRINT_INFO           = SWS_PRINT_INFO as _;
        const FULL_CHR_H_INT       = SWS_FULL_CHR_H_INT as _;
        const FULL_CHR_H_INP       = SWS_FULL_CHR_H_INP as _;
        const DIRECT_BGR           = SWS_DIRECT_BGR as _;
        const ACCURATE_RND         = SWS_ACCURATE_RND as _;
        const BITEXACT             = SWS_BITEXACT as _;
        const ERROR_DIFFUSION      = SWS_ERROR_DIFFUSION as _;
    }
}
