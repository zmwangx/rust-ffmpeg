use libc::c_uint;
use ffi::*;

bitflags! {
	flags Flags: c_uint {
		const UNALIGNED       = CODEC_FLAG_UNALIGNED,
		const QSCALE          = CODEC_FLAG_QSCALE,
		const _4MV            = CODEC_FLAG_4MV,
		const OUTPUT_CORRUPT  = CODEC_FLAG_OUTPUT_CORRUPT,
		const QPEL            = CODEC_FLAG_QPEL,
		const GMC             = CODEC_FLAG_GMC,
		const MV0             = CODEC_FLAG_MV0,
		const INPUT_PRESERVED = CODEC_FLAG_INPUT_PRESERVED,
		const PASS1           = CODEC_FLAG_PASS1,
		const PASS2           = CODEC_FLAG_PASS2,
		const GRAY            = CODEC_FLAG_GRAY,
		const EMU_EDGE        = CODEC_FLAG_EMU_EDGE,
		const PSNR            = CODEC_FLAG_PSNR,
		const TRUNCATED       = CODEC_FLAG_TRUNCATED,
		const NORMALIZE_AQP   = CODEC_FLAG_NORMALIZE_AQP,
		const INTERLACED_DCT  = CODEC_FLAG_INTERLACED_DCT,
		const LOW_DELAY       = CODEC_FLAG_LOW_DELAY,
		const GLOBAL_HEADER   = CODEC_FLAG_GLOBAL_HEADER,
		const BITEXACT        = CODEC_FLAG_BITEXACT,
		const AC_PRED         = CODEC_FLAG_AC_PRED,
		const LOOP_FILTER     = CODEC_FLAG_LOOP_FILTER,
		const INTERLACED_ME   = CODEC_FLAG_INTERLACED_ME,
		const CLOSED_GOP      = CODEC_FLAG_CLOSED_GOP,
	}
}

