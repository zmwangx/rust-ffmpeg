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
            if (*self.as_ptr()).supported_samplerates.is_null() {
                None
            } else {
                Some(RateIter::new((*self.codec.as_ptr()).supported_samplerates))
            }
        }
    }

    pub fn formats(&self) -> Option<FormatIter> {
        unsafe {
            if (*self.codec.as_ptr()).sample_fmts.is_null() {
                None
            } else {
                Some(FormatIter::new((*self.codec.as_ptr()).sample_fmts))
            }
        }
    }

    pub fn channel_layouts(&self) -> Option<ChannelLayoutIter> {
        unsafe {
            if (*self.codec.as_ptr()).channel_layouts.is_null() {
                None
            } else {
                Some(ChannelLayoutIter::new(
                    (*self.codec.as_ptr()).channel_layouts,
                ))
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

pub struct ChannelLayoutIter {
    ptr: *const u64,
}

impl ChannelLayoutIter {
    pub fn new(ptr: *const u64) -> Self {
        ChannelLayoutIter { ptr }
    }

    pub fn best(self, max: i32) -> ChannelLayout {
        self.fold(ChannelLayout::MONO, |acc, cur| {
            if cur.channels() > acc.channels() && cur.channels() <= max {
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
            if *self.ptr == 0 {
                return None;
            }

            let layout = ChannelLayout::from_bits_truncate(*self.ptr);
            self.ptr = self.ptr.offset(1);

            Some(layout)
        }
    }
}
