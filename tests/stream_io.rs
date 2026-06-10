extern crate ffmpeg_next;

use ffmpeg_next::format::context::{Input, Output, StreamIo};
use ffmpeg_next::{format, Error};
use std::io::Cursor;

fn assert_send<T: Send>() {}

#[test]
fn stream_io_and_contexts_are_send() {
    assert_send::<StreamIo>();
    assert_send::<Input>();
    assert_send::<Output>();
}

#[test]
fn into_inner_roundtrip() {
    let io = StreamIo::from_write_seek(Cursor::new(vec![1u8, 2, 3])).unwrap();

    // Wrong type: StreamIo is handed back unchanged.
    let io = io.into_inner::<Vec<u8>>().unwrap_err();

    // Exact construction type: the stream comes back out.
    let cursor = io.into_inner::<Cursor<Vec<u8>>>().expect("type matches");
    assert_eq!(cursor.into_inner(), vec![1, 2, 3]);
}

fn assert_einval<T>(result: Result<T, Error>) {
    let einval = Error::Other {
        errno: ffmpeg_next::util::error::EINVAL,
    };
    match result {
        Err(e) => assert_eq!(e, einval),
        Ok(_) => panic!("expected EINVAL, got Ok"),
    }
}

#[test]
fn invalid_capacity_is_rejected() {
    assert_einval(StreamIo::from_read_with_capacity(Cursor::new(vec![0u8]), 0));
    assert_einval(StreamIo::from_write_with_capacity(
        Vec::new(),
        i32::MAX as usize + 1,
    ));
}

#[test]
fn custom_capacity_roundtrip() {
    let io = StreamIo::from_write_seek_with_capacity(Cursor::new(Vec::new()), 4096).unwrap();
    let cursor = io.into_inner::<Cursor<Vec<u8>>>().expect("type matches");
    assert_eq!(cursor.into_inner(), Vec::<u8>::new());
}

#[test]
fn direction_mismatch_is_rejected() {
    // A write context must not be usable for demuxing.
    let w = StreamIo::from_write(Vec::new()).unwrap();
    assert_einval(format::input_from_stream(w, None, None));

    // A read context must not be usable for muxing.
    let r = StreamIo::from_read(Cursor::new(vec![0u8])).unwrap();
    assert_einval(format::output_to_stream(r, None, Some("matroska")));
}

#[test]
fn nofile_muxers_are_rejected() {
    // image2 (AVFMT_NOFILE) opens one file per frame through its own I/O;
    // `AVFormatContext.pb` is documented to stay NULL for such muxers, so a
    // caller-provided stream would silently never receive the output.
    let w = StreamIo::from_write_seek(Cursor::new(Vec::new())).unwrap();
    assert_einval(format::output_to_stream(
        w,
        Some("frame-%03d.bmp"),
        Some("image2"),
    ));
}

/// Drives the seek callback installed in the `AVIOContext` the way FFmpeg
/// (or a caller invoking `AVIOContext.seek` directly) would.
fn raw_seek(io: &mut StreamIo, offset: i64, whence: i32) -> i64 {
    unsafe {
        let ctx = io.as_mut_ptr();
        ((*ctx).seek.expect("seekable context"))((*ctx).opaque, offset, whence)
    }
}

#[test]
fn seek_masks_avseek_force() {
    use ffmpeg_next::ffi::{AVSEEK_FORCE, AVSEEK_SIZE};

    let mut io = StreamIo::from_read_seek(Cursor::new(vec![0u8; 10])).unwrap();
    // SEEK_SET is 0, so this whence is SEEK_SET | AVSEEK_FORCE.
    assert_eq!(raw_seek(&mut io, 7, AVSEEK_FORCE), 7);
    assert_eq!(raw_seek(&mut io, -2, 2 | AVSEEK_FORCE), 8);
    assert_eq!(raw_seek(&mut io, 0, AVSEEK_SIZE | AVSEEK_FORCE), 10);
    // SEEK_CUR: AVSEEK_SIZE must have restored the position.
    assert_eq!(raw_seek(&mut io, 0, 1), 8);
}

#[test]
fn seek_rejects_negative_absolute_offsets_and_unknown_whence() {
    let einval = ffmpeg_next::ffi::AVERROR(ffmpeg_next::util::error::EINVAL) as i64;

    let mut io = StreamIo::from_read_seek(Cursor::new(vec![0u8; 10])).unwrap();
    assert_eq!(raw_seek(&mut io, -1, 0), einval);
    assert_eq!(raw_seek(&mut io, i64::MIN, 0), einval);
    assert_eq!(raw_seek(&mut io, 0, 3), einval);
    // SEEK_CUR: the failed seeks must not have moved the stream.
    assert_eq!(raw_seek(&mut io, 0, 1), 0);
}

#[test]
fn unrepresentable_positions_are_eoverflow() {
    use std::io::{Read, Seek, SeekFrom};

    // A `Seek` impl is free to report positions `i64` cannot hold; the
    // callback must turn those into an error instead of letting them wrap
    // into the negative AVERROR range.
    struct Huge;
    impl Read for Huge {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Ok(0)
        }
    }
    impl Seek for Huge {
        fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
            Ok(u64::MAX)
        }
    }

    let eoverflow = ffmpeg_next::ffi::AVERROR(ffmpeg_next::util::error::EOVERFLOW) as i64;
    let mut io = StreamIo::from_read_seek(Huge).unwrap();
    assert_eq!(raw_seek(&mut io, 0, 1), eoverflow);
    assert_eq!(
        raw_seek(&mut io, 0, ffmpeg_next::ffi::AVSEEK_SIZE),
        eoverflow
    );
}

#[test]
fn interior_nul_names_error_instead_of_panicking() {
    let r = StreamIo::from_read(Cursor::new(vec![0u8])).unwrap();
    assert_einval(format::input_from_stream(r, Some("bad\0name.mp4"), None));

    let w = StreamIo::from_write(Vec::new()).unwrap();
    assert_einval(format::output_to_stream(w, Some("bad\0name.mp4"), None));
}

/// A minimal but valid WAV file (PCM s16le, mono, 8 kHz) with `data_len`
/// bytes of payload, every payload byte non-zero.
fn tiny_wav(data_len: usize) -> Vec<u8> {
    let mut wav = Vec::with_capacity(44 + data_len);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&8000u32.to_le_bytes()); // sample rate
    wav.extend_from_slice(&16000u32.to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_len as u32).to_le_bytes());
    wav.extend((0..data_len).map(|i| (i % 255) as u8 + 1));
    wav
}

#[test]
fn interrupted_reads_are_retried() {
    use std::io::Read;

    // Yields `ErrorKind::Interrupted` twice before every successful read.
    // FFmpeg has no retry layer above a custom AVIOContext, so unless the
    // wrapper retries these itself, the very first one becomes a sticky
    // error and the open fails.
    struct Interrupting<R> {
        inner: R,
        countdown: u32,
    }
    impl<R: Read> Read for Interrupting<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.countdown > 0 {
                self.countdown -= 1;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            self.countdown = 2;
            self.inner.read(buf)
        }
    }

    let reader = Interrupting {
        inner: Cursor::new(tiny_wav(8192)),
        countdown: 2,
    };
    let mut input = format::input_from_stream(StreamIo::from_read(reader).unwrap(), None, None)
        .expect("interrupted reads must be retried, not surfaced");
    assert_eq!(input.streams().count(), 1);
    assert!(input.packets().count() > 0);
}

#[test]
fn read_buffer_is_handed_to_the_stream_zeroed() {
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // FFmpeg hands the read callback buffers that may be freshly allocated
    // (probe buffers, internal reallocations) or hold stale bytes from
    // earlier fills; the wrapper promises the stream a zeroed slice. A panic
    // would unwind across the C callback boundary, so record violations and
    // assert afterwards.
    struct ZeroCheck {
        inner: Cursor<Vec<u8>>,
        dirty: Arc<AtomicBool>,
    }
    impl Read for ZeroCheck {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if !buf.iter().all(|&b| b == 0) {
                self.dirty.store(true, Ordering::SeqCst);
            }
            self.inner.read(buf)
        }
    }
    impl Seek for ZeroCheck {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    let dirty = Arc::new(AtomicBool::new(false));
    // Large enough that the AVIOContext buffer is refilled (and wrapped)
    // several times after probing, with every payload byte non-zero.
    let reader = ZeroCheck {
        inner: Cursor::new(tiny_wav(200_000)),
        dirty: Arc::clone(&dirty),
    };
    let mut input =
        format::input_from_stream(StreamIo::from_read_seek(reader).unwrap(), None, None).unwrap();
    assert!(input.packets().count() > 0);
    assert!(
        !dirty.load(Ordering::SeqCst),
        "read callback saw a non-zeroed buffer"
    );
}

#[test]
fn nonblocking_and_timed_out_streams_poison_the_context() {
    use std::io::Read;

    struct Failing(std::io::ErrorKind);
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(self.0.into())
        }
    }

    // No retry layer above a custom AVIOContext: the first failure is
    // sticky and surfaces from the open with its truthful errno. The errno
    // must match the `util::error` re-exports users compare against.
    for (kind, errno) in [
        (
            std::io::ErrorKind::WouldBlock,
            ffmpeg_next::util::error::EAGAIN,
        ),
        (
            std::io::ErrorKind::TimedOut,
            ffmpeg_next::util::error::ETIMEDOUT,
        ),
    ] {
        let io = StreamIo::from_read(Failing(kind)).unwrap();
        match format::input_from_stream(io, None, None) {
            Err(e) => assert_eq!(e, Error::Other { errno }),
            Ok(_) => panic!("expected the open to fail"),
        }
    }
}

#[test]
fn custom_io_flag_is_set_on_both_contexts() {
    use ffmpeg_next::ffi::AVFMT_FLAG_CUSTOM_IO;

    let input = format::input_from_stream(
        StreamIo::from_read_seek(Cursor::new(tiny_wav(4096))).unwrap(),
        None,
        None,
    )
    .unwrap();
    assert_ne!(unsafe { (*input.as_ptr()).flags } & AVFMT_FLAG_CUSTOM_IO, 0);

    let output = format::output_to_stream(
        StreamIo::from_write_seek(Cursor::new(Vec::new())).unwrap(),
        None,
        Some("wav"),
    )
    .unwrap();
    assert_ne!(
        unsafe { (*output.as_ptr()).flags } & AVFMT_FLAG_CUSTOM_IO,
        0
    );
}

#[test]
fn writable_stream_is_flushed_on_drop() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct FlushTracker(Arc<AtomicBool>);
    impl std::io::Write for FlushTracker {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    let flushed = Arc::new(AtomicBool::new(false));
    drop(StreamIo::from_write(FlushTracker(Arc::clone(&flushed))).unwrap());
    assert!(flushed.load(Ordering::SeqCst));
}
