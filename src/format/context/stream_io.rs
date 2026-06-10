use ffi;
use libc;
use std::any::TypeId;
use std::convert::TryFrom;
use std::ffi::{c_int, c_void};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::ManuallyDrop;
use Error;

/// Default `AVIOContext` buffer size, matching libavformat's own default.
const DEFAULT_BUFFER_SIZE: usize = 32768;

/// An FFmpeg [`AVIOContext`] backed by a Rust `Read`/`Write`/`Seek` stream,
/// for custom I/O via `format::input_from_stream` / `format::output_to_stream`.
///
/// `StreamIo` owns both the `AVIOContext` and the boxed stream; dropping it
/// frees both. The stream must be `Send + 'static` (the callbacks may run on
/// whatever thread is driving the context), but not `Sync`: callbacks never
/// run concurrently.
///
/// A context is unidirectional: [`StreamIo::from_read`] /
/// [`StreamIo::from_read_seek`] create read (demuxing) contexts,
/// [`StreamIo::from_write`] / [`StreamIo::from_write_seek`] write (muxing)
/// contexts; a mismatch is rejected with `EINVAL`.
///
/// I/O is buffered internally (32 KiB by default; tune it with the
/// `*_with_capacity` constructors). The stream must be *blocking*: FFmpeg has
/// no retry layer for custom I/O, so the first `WouldBlock`/`TimedOut` error
/// poisons the context. `Interrupted` is retried internally, and `Ok(0)` from
/// `read` is reported as EOF.
///
/// Dropping a writable `StreamIo` — directly, or via the `Output` that
/// absorbed it — flushes buffered data and the stream itself, discarding
/// errors (like `std::io::BufWriter`). That keep-alive is shared (an `Arc`)
/// with any `codec::Parameters`/`Context` derived from a stream, so the drop
/// and its flush run on whichever thread releases the last of those owners.
/// For well-formed output you must still call `write_trailer` first. Use
/// [`StreamIo::into_inner`] to get the stream back.
///
/// [`AVIOContext`]: https://ffmpeg.org/doxygen/trunk/structAVIOContext.html
pub struct StreamIo {
    ptr: *mut ffi::AVIOContext,
    drop_opaque: fn(*mut c_void),
    flush_opaque: Option<fn(*mut c_void)>,
    stream_type: TypeId,
}

// SAFETY: every constructor requires the wrapped stream to be `Send`, the
// `AVIOContext` and its buffer are heap allocations not tied to any thread,
// and the stream is only ever accessed through `&mut self` / the callbacks
// (which FFmpeg invokes from the single thread driving the I/O, never
// concurrently — so `Send` without `Sync` is exactly right).
//
// This impl also backs `Send + Sync` on the `Destructor` that embeds a
// `StreamIo` via `destructor::Mode`. A format context's keep-alive is an
// `Arc<Destructor>`, cloned into every stream-derived `codec::{Context,
// Parameters}`; all of those are `Send`, so the last owner to drop — and
// hence `StreamIo::drop`, which flushes and drops the wrapped stream — may
// run on any thread. That is sound precisely because the stream is `Send`.
// `StreamIo` is intentionally not `Sync` and need not be: `Destructor`
// exposes no `&`-access to the embedded `StreamIo`, so `Destructor: Sync`
// (sharing `&Destructor`) can never reach it.
unsafe impl Send for StreamIo {}

impl StreamIo {
    pub fn from_read<T: Read + Send + 'static>(stream: T) -> Result<Self, Error> {
        Self::from_read_with_capacity(stream, DEFAULT_BUFFER_SIZE)
    }
    pub fn from_read_seek<T: Read + Seek + Send + 'static>(stream: T) -> Result<Self, Error> {
        Self::from_read_seek_with_capacity(stream, DEFAULT_BUFFER_SIZE)
    }
    pub fn from_write<T: Write + Send + 'static>(stream: T) -> Result<Self, Error> {
        Self::from_write_with_capacity(stream, DEFAULT_BUFFER_SIZE)
    }
    pub fn from_write_seek<T: Write + Seek + Send + 'static>(stream: T) -> Result<Self, Error> {
        Self::from_write_seek_with_capacity(stream, DEFAULT_BUFFER_SIZE)
    }

    /// Like [`StreamIo::from_read`], with an explicit buffer size in bytes.
    /// Fails with `EINVAL` if `capacity` is zero or exceeds `c_int::MAX`.
    pub fn from_read_with_capacity<T: Read + Send + 'static>(
        stream: T,
        capacity: usize,
    ) -> Result<Self, Error> {
        Self::new_impl(stream, capacity, Some(read::<T>), None, None, None)
    }
    /// Like [`StreamIo::from_read_seek`], with an explicit buffer size in bytes.
    /// Fails with `EINVAL` if `capacity` is zero or exceeds `c_int::MAX`.
    pub fn from_read_seek_with_capacity<T: Read + Seek + Send + 'static>(
        stream: T,
        capacity: usize,
    ) -> Result<Self, Error> {
        Self::new_impl(
            stream,
            capacity,
            Some(read::<T>),
            None,
            Some(seek::<T>),
            None,
        )
    }
    /// Like [`StreamIo::from_write`], with an explicit buffer size in bytes.
    /// Fails with `EINVAL` if `capacity` is zero or exceeds `c_int::MAX`.
    pub fn from_write_with_capacity<T: Write + Send + 'static>(
        stream: T,
        capacity: usize,
    ) -> Result<Self, Error> {
        Self::new_impl(
            stream,
            capacity,
            None,
            Some(write::<T>),
            None,
            Some(flush_stream::<T>),
        )
    }
    /// Like [`StreamIo::from_write_seek`], with an explicit buffer size in bytes.
    /// Fails with `EINVAL` if `capacity` is zero or exceeds `c_int::MAX`.
    pub fn from_write_seek_with_capacity<T: Write + Seek + Send + 'static>(
        stream: T,
        capacity: usize,
    ) -> Result<Self, Error> {
        Self::new_impl(
            stream,
            capacity,
            None,
            Some(write::<T>),
            Some(seek::<T>),
            Some(flush_stream::<T>),
        )
    }

    /// Returns `true` if this is a write (muxing) context.
    pub fn is_writable(&self) -> bool {
        unsafe { (*self.ptr).write_flag != 0 }
    }

    fn new_impl<T: Send + 'static>(
        stream: T,
        capacity: usize,
        r: Option<unsafe extern "C" fn(*mut c_void, *mut u8, c_int) -> c_int>,
        w: Option<unsafe extern "C" fn(*mut c_void, WriteBufferType, c_int) -> c_int>,
        s: Option<unsafe extern "C" fn(*mut c_void, i64, c_int) -> i64>,
        flush: Option<fn(*mut c_void)>,
    ) -> Result<Self, Error> {
        // `AVIOContext::buffer_size` is a C `int`, and a zero-size buffer
        // would make `fill_buffer` / `flush_buffer` spin without progress.
        if capacity == 0 || capacity > c_int::MAX as usize {
            return Err(Error::Other { errno: ffi::EINVAL });
        }
        // Zero-initialized so the slice handed to the first `read` callback
        // never exposes uninitialized memory (see the zeroing in `read` for
        // the buffers FFmpeg itself allocates and swaps in later).
        let buffer = unsafe { ffi::av_mallocz(capacity) };
        if buffer.is_null() {
            return Err(Error::Other { errno: ffi::ENOMEM });
        }
        let stream_box_ptr = Box::into_raw(Box::new(stream)) as *mut c_void;
        let ptr = unsafe {
            ffi::avio_alloc_context(
                buffer as *mut _,
                capacity as _,
                w.is_some() as _,
                stream_box_ptr,
                r,
                w,
                s,
            )
        };
        if ptr.is_null() {
            // `avio_alloc_context` takes ownership of `buffer` only on success.
            unsafe {
                ffi::av_free(buffer);
                drop(Box::from_raw(stream_box_ptr as *mut T));
            }
            return Err(Error::Other { errno: ffi::ENOMEM });
        }

        Ok(Self {
            ptr,
            drop_opaque: drop_box::<T>,
            flush_opaque: flush,
            stream_type: TypeId::of::<T>(),
        })
    }

    /// Consumes the `StreamIo` and returns the wrapped stream, flushing data
    /// still buffered in the `AVIOContext` first. The stream's own
    /// [`Write::flush`] is *not* called, so the caller can flush and observe
    /// errors. Fails (returning `self`) unless `T` is the exact type the
    /// `StreamIo` was constructed with.
    pub fn into_inner<T: 'static>(self) -> Result<T, Self> {
        if self.stream_type != TypeId::of::<T>() {
            return Err(self);
        }
        let mut this = ManuallyDrop::new(self);
        unsafe {
            ffi::avio_flush(this.ptr);
            let opaque = (*this.ptr).opaque;
            ffi::av_freep(&raw mut (*this.ptr).buffer as *mut c_void);
            ffi::avio_context_free(&mut this.ptr);
            Ok(*Box::from_raw(opaque as *mut T))
        }
    }

    /// Returns a mutable raw pointer to the underlying `AVIOContext`.
    ///
    /// # Safety
    /// The returned pointer is owned by `self`. Do **not** free it or mutate its
    /// `buffer`/`opaque` fields directly. It must not outlive `self`.
    pub fn as_mut_ptr(&mut self) -> *mut ffi::AVIOContext {
        self.ptr
    }
}

impl Drop for StreamIo {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let opaque = (*self.ptr).opaque;
                if (*self.ptr).write_flag != 0 {
                    // Salvage data still buffered in the AVIOContext, then let
                    // the stream flush its own buffers (a user `BufWriter`
                    // tail would otherwise only flush in its Drop, or be lost).
                    // Errors are unreportable from a destructor and are
                    // discarded, like `std::io::BufWriter` does.
                    ffi::avio_flush(self.ptr);
                    if let Some(flush) = self.flush_opaque {
                        flush(opaque);
                    }
                }
                ffi::av_freep(&raw mut (*self.ptr).buffer as *mut c_void);
                ffi::avio_context_free(&mut self.ptr);
                (self.drop_opaque)(opaque);
            }
        }
    }
}

impl std::fmt::Debug for StreamIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamIo").field("ptr", &self.ptr).finish()
    }
}

unsafe extern "C" fn read<T: Read>(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int {
    // FFmpeg never issues zero-sized reads (and `read_packet` must not
    // return 0 — it asserts on that), but a `Read` impl would report one as
    // `Ok(0)`, which we translate to EOF; reject instead of lying.
    if buf_size <= 0 {
        return ffi::AVERROR(ffi::EINVAL);
    }
    // `buf` routinely points into uninitialized memory FFmpeg allocated: the
    // probe buffer `ffio_rewind_with_probe_data` swaps in as `s->buffer`, the
    // post-probe `set_buf_size` reallocation, and direct reads into caller
    // buffers all come from plain `av_malloc`/`av_realloc`. A safe `Read`
    // impl is allowed to read from the slice it is given, so it must be
    // initialized; the memset is noise next to the I/O itself.
    unsafe { std::ptr::write_bytes(buf, 0, buf_size as usize) };
    let buf = unsafe { std::slice::from_raw_parts_mut(buf, buf_size as usize) };
    let stream = unsafe { &mut *(opaque as *mut T) };
    loop {
        return match stream.read(buf) {
            Ok(0) => ffi::AVERROR_EOF,
            // A buggy (but safe) `Read` impl may report more bytes than the buffer
            // holds; FFmpeg trusts the count and would advance `buf_end` past the
            // allocation.
            Ok(n) if n > buf.len() => ffi::AVERROR(ffi::EIO),
            Ok(n) => n as c_int,
            // Retry interrupted reads like FFmpeg's own protocol layer does;
            // surfacing EINTR would poison the context (see `map_io_error`)
            // even though the read can simply be reissued.
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => map_io_error(e),
        };
    }
}
unsafe extern "C" fn write<T: Write>(
    opaque: *mut c_void,
    buf: WriteBufferType,
    buf_size: c_int,
) -> c_int {
    if buf_size < 0 {
        return ffi::AVERROR(ffi::EINVAL);
    }
    let buf = unsafe { std::slice::from_raw_parts(buf, buf_size as usize) };
    let stream = unsafe { &mut *(opaque as *mut T) };
    // FFmpeg treats any non-negative return as "the whole buffer was written"
    // and never retries a remainder, so a short `write` would silently lose
    // data; `write_all` is the only faithful mapping. It also retries
    // `ErrorKind::Interrupted` internally (per its contract), so EINTR never
    // reaches `map_io_error` — mirroring the retry loop in `read`.
    match stream.write_all(buf) {
        Ok(()) => buf_size,
        Err(e) => map_io_error(e),
    }
}
unsafe extern "C" fn seek<T: Seek>(opaque: *mut c_void, offset: i64, whence: c_int) -> i64 {
    let stream = unsafe { &mut *(opaque as *mut T) };

    // AVSEEK_FORCE may be OR'd into `whence` ("seek by any means"); avio.h
    // documents it as ignored by the seek code since 2010, and FFmpeg's own
    // dispatchers mask it off before seeking (`avio_seek`, `ffurl_seek`).
    // Honor the flag convention instead of failing the seek with EINVAL.
    let whence = whence & !ffi::AVSEEK_FORCE;

    if whence == ffi::AVSEEK_SIZE {
        // Return the stream size. Any negative return makes `avio_size` fall
        // back to probing with SEEK_END, which also restores the position
        // FFmpeg expects, so a partial failure here cannot corrupt state.
        match stream.stream_position().and_then(|cur| {
            let end = stream.seek(SeekFrom::End(0))?;
            if cur != end {
                stream.seek(SeekFrom::Start(cur))?;
            }
            Ok(end)
        }) {
            Ok(sz) => return position_to_i64(sz),
            Err(e) => return map_io_error(e) as i64,
        }
    }

    let pos = match whence {
        // `avio_seek` rejects negative absolute offsets before invoking the
        // callback, so one can only arrive from a caller driving the callback
        // directly; `as u64` would turn it into a huge forward seek.
        0 if offset >= 0 => SeekFrom::Start(offset as u64),
        0 => return ffi::AVERROR(ffi::EINVAL) as i64,
        1 => SeekFrom::Current(offset),
        2 => SeekFrom::End(offset),
        _ => return ffi::AVERROR(ffi::EINVAL) as i64,
    };
    match stream.seek(pos) {
        Ok(pos) => position_to_i64(pos),
        Err(e) => map_io_error(e) as i64,
    }
}

// `Seek` reports positions as `u64`, but the callback returns `i64` with
// negative values reserved for AVERROR codes; a position above `i64::MAX`
// would wrap into (or alias) an error code.
fn position_to_i64(pos: u64) -> i64 {
    i64::try_from(pos).unwrap_or(ffi::AVERROR(libc::EOVERFLOW) as i64)
}

// Not a C callback: invoked from `StreamIo::drop` to flush the user stream
// after the `AVIOContext` buffer has been written out.
fn flush_stream<T: Write>(opaque: *mut c_void) {
    let _ = unsafe { &mut *(opaque as *mut T) }.flush();
}

// Not a C callback: invoked from `StreamIo::drop` to free the boxed stream.
// `opaque` must be the `Box<T>` created in `new_impl`.
fn drop_box<T>(opaque: *mut c_void) {
    drop(unsafe { Box::from_raw(opaque as *mut T) });
}

fn map_io_error(e: std::io::Error) -> i32 {
    use std::io::ErrorKind::*;
    // On Unix the raw OS error *is* an errno value, which is exactly what
    // AVERROR encodes; pass it through to preserve detail (EACCES, ENOSPC,
    // ...). On Windows it is a Win32 error code, not an errno, so it cannot
    // be used and we fall back to mapping the `ErrorKind`.
    #[cfg(unix)]
    if let Some(errno) = e.raw_os_error() {
        if errno > 0 {
            return ffi::AVERROR(errno);
        }
    }
    // Errors returned from the read/write callbacks are sticky: there is no
    // retry layer above a custom AVIOContext (FFmpeg retries EINTR/EAGAIN
    // only inside its own URL protocols), so `fill_buffer`/`writeout` latch
    // whatever we return into `s->error` and no further I/O happens. That is
    // why `Interrupted` is retried in the callbacks instead of being mapped
    // here (this arm stays reachable from `seek`, which FFmpeg does not
    // treat as sticky), and why `WouldBlock`/`TimedOut` — while given their
    // truthful codes — are fatal: see the "Blocking I/O" notes on `StreamIo`.
    //
    // The errno constants come from `libc`, not the generated bindings: they
    // must agree with the `util::error` re-exports users match `Error::Other`
    // against (and with the platform CRT the FFmpeg binary itself was built
    // with), whereas bindgen has been observed emitting glibc values on
    // Windows (ETIMEDOUT 110 vs the CRT's 138).
    match e.kind() {
        UnexpectedEof => ffi::AVERROR_EOF,
        Interrupted => ffi::AVERROR(libc::EINTR),
        WouldBlock => ffi::AVERROR(libc::EAGAIN),
        TimedOut => ffi::AVERROR(libc::ETIMEDOUT),
        Unsupported => ffi::AVERROR(libc::ENOSYS),
        _ => ffi::AVERROR(libc::EIO),
    }
}

#[cfg(not(feature = "ffmpeg_7_0"))]
type WriteBufferType = *mut u8;

#[cfg(feature = "ffmpeg_7_0")]
type WriteBufferType = *const u8;
