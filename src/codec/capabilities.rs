use ffi::*;
use libc::c_uint;

bitflags! {
    pub struct Capabilities: c_uint {
        const DRAW_HORIZ_BAND     = AV_CODEC_CAP_DRAW_HORIZ_BAND;
        const DR1                 = AV_CODEC_CAP_DR1;
        const TRUNCATED           = AV_CODEC_CAP_TRUNCATED;
        const DELAY               = AV_CODEC_CAP_DELAY;
        const SMALL_LAST_FRAME    = AV_CODEC_CAP_SMALL_LAST_FRAME;
        #[cfg(not(feature = "ffmpeg4"))]
        const HWACCEL_VDPAU       = AV_CODEC_CAP_HWACCEL_VDPAU;
        const SUBFRAMES           = AV_CODEC_CAP_SUBFRAMES;
        const EXPERIMENTAL        = AV_CODEC_CAP_EXPERIMENTAL;
        const CHANNEL_CONF        = AV_CODEC_CAP_CHANNEL_CONF;
        const FRAME_THREADS       = AV_CODEC_CAP_FRAME_THREADS;
        const SLICE_THREADS       = AV_CODEC_CAP_SLICE_THREADS;
        const PARAM_CHANGE        = AV_CODEC_CAP_PARAM_CHANGE;
        const AUTO_THREADS        = AV_CODEC_CAP_AUTO_THREADS;
        const VARIABLE_FRAME_SIZE = AV_CODEC_CAP_VARIABLE_FRAME_SIZE;
        const INTRA_ONLY          = AV_CODEC_CAP_INTRA_ONLY;
        const LOSSLESS            = AV_CODEC_CAP_LOSSLESS;
    }
}
