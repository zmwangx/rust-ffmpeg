use libc::c_uint;
use ffi::*;

bitflags! {
	flags Capabilities: c_uint {
		const MMX     = SWS_CPU_CAPS_MMX,
		const MMXEXT  = SWS_CPU_CAPS_MMXEXT,
		const MMX2    = SWS_CPU_CAPS_MMX2,
		const _3DNOW  = SWS_CPU_CAPS_3DNOW,
		const ALTIVEC = SWS_CPU_CAPS_ALTIVEC,
		const BFIN    = SWS_CPU_CAPS_BFIN,
		const SSE2    = SWS_CPU_CAPS_SSE2,
	}
}
