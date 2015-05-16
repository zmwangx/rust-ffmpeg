use libc::c_int;
use ffi::*;

bitflags! {
	flags Debug: c_int {
		const DEBUG_PICT_INFO   = FF_DEBUG_PICT_INFO,
		const DEBUG_RC          = FF_DEBUG_RC,
		const DEBUG_BITSTREAM   = FF_DEBUG_BITSTREAM,
		const DEBUG_MB_TYPE     = FF_DEBUG_MB_TYPE,
		const DEBUG_QP          = FF_DEBUG_QP,
		const DEBUG_MV          = FF_DEBUG_MV,
		const DEBUG_DCT_COEFF   = FF_DEBUG_DCT_COEFF,
		const DEBUG_SKIP        = FF_DEBUG_SKIP,
		const DEBUG_STARTCODE   = FF_DEBUG_STARTCODE,
		const DEBUG_PTS         = FF_DEBUG_PTS,
		const DEBUG_ER          = FF_DEBUG_ER,
		const DEBUG_MMCO        = FF_DEBUG_MMCO,
		const DEBUG_BUGS        = FF_DEBUG_BUGS,
		const DEBUG_VIS_QP      = FF_DEBUG_VIS_QP,
		const DEBUG_VIS_MB_TYPE = FF_DEBUG_VIS_MB_TYPE,
		const DEBUG_BUFFERS     = FF_DEBUG_BUFFERS,
		const DEBUG_THREADS     = FF_DEBUG_THREADS,
		const DEBUG_NOMC        = FF_DEBUG_NOMC,
	}
}
