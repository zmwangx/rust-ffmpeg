use ffi::*;
use libc::c_int;

bitflags! {
    pub struct Debug: c_int {
        const PICT_INFO   = FF_DEBUG_PICT_INFO;
        const RC          = FF_DEBUG_RC;
        const BITSTREAM   = FF_DEBUG_BITSTREAM;
        const MB_TYPE     = FF_DEBUG_MB_TYPE;
        const QP          = FF_DEBUG_QP;
        const DCT_COEFF   = FF_DEBUG_DCT_COEFF;
        const SKIP        = FF_DEBUG_SKIP;
        const STARTCODE   = FF_DEBUG_STARTCODE;
        const ER          = FF_DEBUG_ER;
        const MMCO        = FF_DEBUG_MMCO;
        const BUGS        = FF_DEBUG_BUGS;
        const BUFFERS     = FF_DEBUG_BUFFERS;
        const THREADS     = FF_DEBUG_THREADS;
        const NOMC        = FF_DEBUG_NOMC;
    }
}
