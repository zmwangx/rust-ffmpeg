pub mod flag;
pub use self::flag::Flags;

pub mod id;
pub use self::id::Id;

pub mod packet;

pub mod subtitle;

#[cfg(not(feature = "ffmpeg_5_0"))]
pub mod picture;

pub mod discard;

pub mod context;
pub use self::context::Context;

pub mod capabilities;
pub use self::capabilities::Capabilities;

pub mod codec;
pub use self::codec::Iter;

pub mod parameters;
pub use self::parameters::Parameters;

pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

pub mod audio_service;
pub mod field_order;

pub mod compliance;
pub use self::compliance::Compliance;

pub mod debug;
pub use self::debug::Debug;

pub mod profile;
pub use self::profile::Profile;

pub mod threading;

pub mod decoder;
pub mod encoder;
pub mod traits;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use crate::ffi::*;

/// Query a codec's supported configuration list via `avcodec_get_supported_config`.
#[cfg(feature = "ffmpeg_9_0")]
pub(crate) unsafe fn supported_config<T>(codec: *const AVCodec, config: AVCodecConfig) -> *const T {
    unsafe {
        let mut out: *const T = std::ptr::null();
        avcodec_get_supported_config(
            std::ptr::null(),
            codec,
            config,
            0,
            (&mut out as *mut *const T).cast(),
            std::ptr::null_mut(),
        );
        out
    }
}

pub fn version() -> u32 {
    unsafe { avcodec_version() }
}

pub fn configuration() -> &'static str {
    unsafe { from_utf8_unchecked(CStr::from_ptr(avcodec_configuration()).to_bytes()) }
}

pub fn license() -> &'static str {
    unsafe { from_utf8_unchecked(CStr::from_ptr(avcodec_license()).to_bytes()) }
}

pub fn list() -> Iter {
    Iter::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_codecs() {
        assert!(list().any(|codec| codec.is_encoder() || codec.is_decoder()));
    }
}
