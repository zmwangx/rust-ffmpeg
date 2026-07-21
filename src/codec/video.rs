use std::ops::Deref;

use super::codec::Codec;
use crate::ffi::*;
use crate::{Rational, format};

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Video {
    codec: Codec,
}

impl Video {
    pub unsafe fn new(codec: Codec) -> Video {
        Video { codec }
    }
}

impl Video {
    pub fn rates(&self) -> Option<RateIter> {
        unsafe {
            #[cfg(feature = "ffmpeg_9_0")]
            let ptr = super::supported_config::<AVRational>(
                self.codec.as_ptr(),
                AVCodecConfig::AV_CODEC_CONFIG_FRAME_RATE,
            );
            #[cfg(not(feature = "ffmpeg_9_0"))]
            let ptr = (*self.codec.as_ptr()).supported_framerates;

            if ptr.is_null() {
                None
            } else {
                Some(RateIter::new(ptr))
            }
        }
    }

    pub fn formats(&self) -> Option<FormatIter> {
        unsafe {
            #[cfg(feature = "ffmpeg_9_0")]
            let ptr = super::supported_config::<AVPixelFormat>(
                self.codec.as_ptr(),
                AVCodecConfig::AV_CODEC_CONFIG_PIX_FORMAT,
            );
            #[cfg(not(feature = "ffmpeg_9_0"))]
            let ptr = (*self.codec.as_ptr()).pix_fmts;

            if ptr.is_null() {
                None
            } else {
                Some(FormatIter::new(ptr))
            }
        }
    }
}

impl Deref for Video {
    type Target = Codec;

    fn deref(&self) -> &Self::Target {
        &self.codec
    }
}

pub struct RateIter {
    ptr: *const AVRational,
}

impl RateIter {
    pub fn new(ptr: *const AVRational) -> Self {
        RateIter { ptr }
    }
}

impl Iterator for RateIter {
    type Item = Rational;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            if (*self.ptr).num == 0 && (*self.ptr).den == 0 {
                return None;
            }

            let rate = (*self.ptr).into();
            self.ptr = self.ptr.offset(1);

            Some(rate)
        }
    }
}

pub struct FormatIter {
    ptr: *const AVPixelFormat,
}

impl FormatIter {
    pub fn new(ptr: *const AVPixelFormat) -> Self {
        FormatIter { ptr }
    }
}

impl Iterator for FormatIter {
    type Item = format::Pixel;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            if *self.ptr == AVPixelFormat::AV_PIX_FMT_NONE {
                return None;
            }

            let format = (*self.ptr).into();
            self.ptr = self.ptr.offset(1);

            Some(format)
        }
    }
}
