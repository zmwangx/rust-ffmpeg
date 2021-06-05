use std::ffi::CString;
use std::mem;
use std::ops::{Deref, DerefMut};

use super::common::Context;
use super::destructor;
use ffi::*;
use util::range::Range;
use {format, Codec, Error, Packet, Stream};
use libc::c_int;


bitflags! {
    pub struct SeekFlags: c_int {
        const ANY = sys::AVSEEK_FLAG_ANY;
        const BACKWARD = sys::AVSEEK_FLAG_BACKWARD;
        const BYTE = sys::AVSEEK_FLAG_BYTE;
        const FRAME = sys::AVSEEK_FLAG_FRAME;
    }
}

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
        unsafe { format::Input::wrap((*self.as_ptr()).iformat) }
    }

    pub fn video_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = av_format_get_video_codec(self.as_ptr());

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    pub fn audio_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = av_format_get_audio_codec(self.as_ptr());

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    pub fn subtitle_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = av_format_get_subtitle_codec(self.as_ptr());

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    pub fn data_codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = av_format_get_data_codec(self.as_ptr());

            if ptr.is_null() {
                None
            } else {
                Some(Codec::wrap(ptr))
            }
        }
    }

    pub fn probe_score(&self) -> i32 {
        unsafe { av_format_get_probe_score(self.as_ptr()) }
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

    /// Seeks to a specific frame in the input.
    ///
    /// For more info view [ffmpeg's documentation](
    /// https://ffmpeg.org/doxygen/3.4/group__lavf__decoding.html#gaa23f7619d8d4ea0857065d9979c75ac8).
    pub fn seek_to_frame(&mut self, stream_idx: i32, timestamp: i64, flags: SeekFlags
            ) -> Result<(), Error> {
        unsafe {
            match sys::av_seek_frame(self.as_mut_ptr(), stream_idx, timestamp, flags.bits())
            {
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
