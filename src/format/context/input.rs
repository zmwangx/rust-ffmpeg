use std::ffi::CString;
use std::mem;
use std::ops::{Deref, DerefMut};

use super::common::Context;
use super::destructor;
use ffi::*;
use util::range::Range;
#[cfg(not(feature = "ffmpeg_5_0"))]
use Codec;
use {format, Error, Packet, Stream};

pub struct Input {
    ptr: *mut AVFormatContext,
    ctx: Context,
}

unsafe impl Send for Input {}

impl Input {
    pub unsafe fn wrap(ptr: *mut AVFormatContext) -> Self {
        Input {
            ptr,
            ctx: Context::wrap(ptr, destructor::Mode::Input),
        }
    }

    pub unsafe fn as_ptr(&self) -> *const AVFormatContext {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
        self.ptr
    }
}

impl Input {
    pub fn format(&self) -> format::Input {
        // We get a clippy warning in 4.4 but not in 5.0 and newer, so we allow that cast to not complicate the code
        #[allow(clippy::unnecessary_cast)]
        unsafe {
            format::Input::wrap((*self.as_ptr()).iformat as *mut AVInputFormat)
        }
    }

    #[cfg(not(feature = "ffmpeg_5_0"))]
    pub fn video_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = (*self.as_ptr()).video_codec;

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    #[cfg(not(feature = "ffmpeg_5_0"))]
    pub fn audio_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = (*self.as_ptr()).audio_codec;

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    #[cfg(not(feature = "ffmpeg_5_0"))]
    pub fn subtitle_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = (*self.as_ptr()).subtitle_codec;

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    #[cfg(not(feature = "ffmpeg_5_0"))]
    pub fn data_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = (*self.as_ptr()).data_codec;

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    pub fn probe_score(&self) -> i32 {
        unsafe { (*self.as_ptr()).probe_score }
    }

    pub fn packets(&mut self) -> PacketIter {
        PacketIter::new(self)
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        unsafe {
            match av_read_pause(self.as_mut_ptr()) {
                0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }

    pub fn play(&mut self) -> Result<(), Error> {
        unsafe {
            match av_read_play(self.as_mut_ptr()) {
                0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }

    pub fn seek<R: Range<i64>>(&mut self, ts: i64, range: R) -> Result<(), Error> {
        unsafe {
            match avformat_seek_file(
                self.as_mut_ptr(),
                -1,
                range.start().cloned().unwrap_or(i64::min_value()),
                ts,
                range.end().cloned().unwrap_or(i64::max_value()),
                0,
            ) {
                s if s >= 0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }
}

impl Deref for Input {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for Input {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

pub struct PacketIter<'a> {
    context: &'a mut Input,
}

impl<'a> PacketIter<'a> {
    pub fn new(context: &mut Input) -> PacketIter {
        PacketIter { context }
    }
}

impl<'a> Iterator for PacketIter<'a> {
    type Item = (Stream<'a>, Packet);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut packet = Packet::empty();

        loop {
            match packet.read(self.context) {
                Ok(..) => unsafe {
                    return Some((
                        Stream::wrap(mem::transmute_copy(&self.context), packet.stream()),
                        packet,
                    ));
                },

                Err(Error::Eof) => return None,

                Err(..) => (),
            }
        }
    }
}

pub fn dump(ctx: &Input, index: i32, url: Option<&str>) {
    let url = url.map(|u| CString::new(u).unwrap());

    unsafe {
        av_dump_format(
            ctx.as_ptr() as *mut _,
            index,
            url.unwrap_or_else(|| CString::new("").unwrap()).as_ptr(),
            0,
        );
    }
}
