use std::ops::Deref;

use super::codec::Codec;
use ffi::*;
use {format, ChannelLayout};

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Audio {
    codec: Codec,
}

impl Audio {
    pub unsafe fn new(codec: Codec) -> Audio {
        Audio { codec }
    }
}

impl Audio {
    pub fn rates(&self) -> Option<RateIter> {
        unsafe {
            #[cfg(feature = "ffmpeg_9_0")]
            let ptr = super::supported_config::<i32>(
                self.codec.as_ptr(),
                AVCodecConfig::AV_CODEC_CONFIG_SAMPLE_RATE,
            );
            #[cfg(not(feature = "ffmpeg_9_0"))]
            let ptr = (*self.codec.as_ptr()).supported_samplerates;

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
            let ptr = super::supported_config::<AVSampleFormat>(
                self.codec.as_ptr(),
                AVCodecConfig::AV_CODEC_CONFIG_SAMPLE_FORMAT,
            );
            #[cfg(not(feature = "ffmpeg_9_0"))]
            let ptr = (*self.codec.as_ptr()).sample_fmts;

            if ptr.is_null() {
                None
            } else {
                Some(FormatIter::new(ptr))
            }
        }
    }

    pub fn channel_layouts(&self) -> Option<ChannelLayoutIter> {
        unsafe {
            #[cfg(not(feature = "ffmpeg_7_0"))]
            let ptr = (*self.codec.as_ptr()).channel_layouts;

            #[cfg(all(feature = "ffmpeg_7_0", not(feature = "ffmpeg_9_0")))]
            let ptr = (*self.codec.as_ptr()).ch_layouts;

            #[cfg(feature = "ffmpeg_9_0")]
            let ptr = super::supported_config::<AVChannelLayout>(
                self.codec.as_ptr(),
                AVCodecConfig::AV_CODEC_CONFIG_CHANNEL_LAYOUT,
            );

            if ptr.is_null() {
                None
            } else {
                Some(ChannelLayoutIter::new(ptr))
            }
        }
    }
}

impl Deref for Audio {
    type Target = Codec;

    fn deref(&self) -> &Self::Target {
        &self.codec
    }
}

pub struct RateIter {
    ptr: *const i32,
}

impl RateIter {
    pub fn new(ptr: *const i32) -> Self {
        RateIter { ptr }
    }
}

impl Iterator for RateIter {
    type Item = i32;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            if *self.ptr == 0 {
                return None;
            }

            let rate = *self.ptr;
            self.ptr = self.ptr.offset(1);

            Some(rate)
        }
    }
}

pub struct FormatIter {
    ptr: *const AVSampleFormat,
}

impl FormatIter {
    pub fn new(ptr: *const AVSampleFormat) -> Self {
        FormatIter { ptr }
    }
}

impl Iterator for FormatIter {
    type Item = format::Sample;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            if *self.ptr == AVSampleFormat::AV_SAMPLE_FMT_NONE {
                return None;
            }

            let format = (*self.ptr).into();
            self.ptr = self.ptr.offset(1);

            Some(format)
        }
    }
}

#[cfg(not(feature = "ffmpeg_7_0"))]
type ChannelLayoutType = u64;
#[cfg(feature = "ffmpeg_7_0")]
type ChannelLayoutType = AVChannelLayout;

pub struct ChannelLayoutIter {
    ptr: *const ChannelLayoutType,
}

impl ChannelLayoutIter {
    pub fn new(ptr: *const ChannelLayoutType) -> Self {
        ChannelLayoutIter { ptr }
    }

    pub fn best(self, max: i32) -> ChannelLayout {
        self.fold(ChannelLayout::MONO, |acc, cur| {
            if cur.channels() > acc.channels() && cur.channels() <= max as _ {
                cur
            } else {
                acc
            }
        })
    }
}

impl Iterator for ChannelLayoutIter {
    type Item = ChannelLayout;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        unsafe {
            #[cfg(not(feature = "ffmpeg_7_0"))]
            if *self.ptr == 0 {
                return None;
            }

            #[cfg(feature = "ffmpeg_7_0")]
            if self.ptr.is_null() || (*self.ptr).u.mask == 0 {
                return None;
            }

            #[cfg(not(feature = "ffmpeg_7_0"))]
            let layout = ChannelLayout::from_bits_truncate(*self.ptr);

            #[cfg(feature = "ffmpeg_7_0")]
            let layout = ChannelLayout::from(*self.ptr);

            self.ptr = self.ptr.offset(1);

            Some(layout)
        }
    }
}
