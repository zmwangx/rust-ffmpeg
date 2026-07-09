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
    pub unsafe fn wrap_with_custom_io(
        ptr: *mut AVFormatContext,
        custom_io: format::context::StreamIo,
    ) -> Self {
        Input {
            ptr,
            ctx: Context::wrap(ptr, destructor::Mode::InputCustomIo(custom_io)),
        }
    }

    pub unsafe fn wrap_with_interrupt(
        ptr: *mut AVFormatContext,
        guard: ::util::interrupt::InterruptGuard,
    ) -> Self {
        Input {
            ptr,
            ctx: Context::wrap_with_interrupt(ptr, destructor::Mode::Input, guard),
        }
    }

    pub unsafe fn wrap_with_custom_io_and_interrupt(
        ptr: *mut AVFormatContext,
        custom_io: format::context::StreamIo,
        guard: ::util::interrupt::InterruptGuard,
    ) -> Self {
        Input {
            ptr,
            ctx: Context::wrap_with_interrupt(
                ptr,
                destructor::Mode::InputCustomIo(custom_io),
                guard,
            ),
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

    pub fn packets(&mut self) -> PacketIter<'_> {
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
            let pb = (*self.ptr).pb;
            // Clear the latch BEFORE seeking: the seek machinery itself gates
            // on `eof_reached`/`error`, so a "clear only on success" ordering
            // cannot work.
            let relatch = unlatch_exit(pb);
            let ret = avformat_seek_file(
                self.as_mut_ptr(),
                -1,
                range.start().cloned().unwrap_or(i64::MIN),
                ts,
                range.end().cloned().unwrap_or(i64::MAX),
                0,
            );
            if ret < 0 && relatch {
                // The seek failed after we cleared the latch — and after
                // `avformat_seek_file` already flushed/reset demuxer state
                // (e.g. EPIPE on a non-seekable `from_read` stream). Re-poison
                // the session so the next read fails loudly with `Error::Exit`
                // rather than resyncing into silent data skips: `eof_reached=1`
                // makes the next read short-circuit to EOF, which
                // `read_frame_internal` rewrites back into the sticky EXIT.
                (*pb).error = AVERROR_EXIT;
                (*pb).eof_reached = 1;
            }
            match ret {
                s if s >= 0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }

    pub fn io_size(&self) -> Option<i64> {
        unsafe {
            let pb = (*self.as_ptr()).pb;
            if pb.is_null() {
                return None;
            }
            let sz = avio_size(pb);
            if sz < 0 {
                None
            } else {
                Some(sz)
            }
        }
    }

    pub fn clear_eof(&mut self) -> bool {
        unsafe {
            let pb = (*self.as_ptr()).pb;
            if pb.is_null() {
                return false;
            }
            (*pb).eof_reached = 0;
            (*pb).error = 0;
            true
        }
    }

    /// Clears a pending interrupt-callback abort (`AVERROR_EXIT`) latched into
    /// the `AVIOContext` by a cancelled blocking read, returning `true` if one
    /// was cleared. Unlike [`clear_eof`](Self::clear_eof), a genuine sticky I/O
    /// error is preserved.
    ///
    /// This is the post-cancel resume point for a **non-seekable** stream (a
    /// `from_read` stream whose [`seek`](Self::seek) would fail with EPIPE):
    /// re-arm the interrupt token, call this, then keep reading forward.
    pub fn clear_interrupt(&mut self) -> bool {
        unsafe { unlatch_exit((*self.ptr).pb) }
    }
}

/// Un-latch a prior interrupt-callback abort (`AVERROR_EXIT`) from an
/// `AVIOContext`, returning whether one was latched. A genuine sticky I/O error
/// (any other `error` value) is preserved. Callers that must keep the abort
/// poisoned on a later failure re-latch it (see [`Input::seek`]).
///
/// # Safety
/// `pb` must be null or a valid `AVIOContext` owned by the format context.
unsafe fn unlatch_exit(pb: *mut AVIOContext) -> bool {
    if pb.is_null() || (*pb).error != AVERROR_EXIT {
        return false;
    }
    (*pb).error = 0;
    (*pb).eof_reached = 0;
    true
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
    pub fn new(context: &mut Input) -> PacketIter<'_> {
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

                // Skip a single corrupt packet and keep demuxing: a demuxer can
                // resync past `AVERROR_INVALIDDATA`, and it is not latched into
                // the `AVIOContext` (`pb->error`), so retrying makes progress.
                Err(Error::InvalidData) => (),

                // Every other error is terminal. A cancelled read's
                // `AVERROR_EXIT`, or any I/O error, is latched into `pb->error`
                // (aviobuf.c `fill_buffer`) and `av_read_frame` then returns it
                // on EVERY subsequent call (demux.c rewrites even a later clean
                // EOF back into the sticky error), so retrying would spin
                // forever at 100% CPU. End the iteration instead. Callers that
                // must observe these errors drive `Packet::read` directly.
                Err(..) => return None,
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
