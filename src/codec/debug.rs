use libc::c_int;
use ffi::*;

bitflags! {
	flags Debug: c_int {
		const PICT_INFO   = FF_DEBUG_PICT_INFO,
		const RC          = FF_DEBUG_RC,
		const BITSTREAM   = FF_DEBUG_BITSTREAM,
		const MB_TYPE     = FF_DEBUG_MB_TYPE,
		const QP          = FF_DEBUG_QP,
		const MV          = FF_DEBUG_MV,
		const DCT_COEFF   = FF_DEBUG_DCT_COEFF,
		const SKIP        = FF_DEBUG_SKIP,
		const STARTCODE   = FF_DEBUG_STARTCODE,
		const PTS         = FF_DEBUG_PTS,
		const ER          = FF_DEBUG_ER,
		const MMCO        = FF_DEBUG_MMCO,
		const BUGS        = FF_DEBUG_BUGS,
		const VIS_QP      = FF_DEBUG_VIS_QP,
		const VIS_MB_TYPE = FF_DEBUG_VIS_MB_TYPE,
		const BUFFERS     = FF_DEBUG_BUFFERS,
		const THREADS     = FF_DEBUG_THREADS,
		const NOMC        = FF_DEBUG_NOMC,
	}
}
