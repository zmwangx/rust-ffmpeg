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
fn armed_but_unfired_interrupt_retries_transient_interrupted() {
    use std::io::Read;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // With an interrupt callback INSTALLED but its token unfired, a transient
    // `Interrupted` must still be retried, not aborted: the callbacks poll the
    // token at the top of each attempt and abort only when it reads `true`.
    // Regression guard for a callback that aborts on *any* `Interrupted`
    // whenever a CB is present (instead of actually consulting it) — that would
    // turn these transient EINTRs into a spurious `Error::Exit` at open.
    struct TransientInterrupt<R> {
        inner: R,
        remaining: u32,
    }
    impl<R: Read> Read for TransientInterrupt<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.remaining > 0 {
                self.remaining -= 1;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            self.inner.read(buf)
        }
    }

    let token = Arc::new(AtomicBool::new(false)); // armed, never fired
    let cb_token = Arc::clone(&token);
    let reader = TransientInterrupt {
        inner: Cursor::new(tiny_wav(8192)),
        remaining: 3,
    };
    let mut input = format::input_from_stream_with_interrupt(
        StreamIo::from_read(reader).unwrap(),
        None,
        None,
        move || cb_token.load(Ordering::SeqCst),
    )
    .expect("transient Interrupted must be retried while the token is unfired");
    assert_eq!(input.streams().count(), 1);
    assert!(input.packets().count() > 0);
}

#[test]
fn armed_interrupt_aborts_during_open() {
    use std::io::Read;

    // The interrupt callback must be installed (and mirrored into the StreamIo
    // opaque) BEFORE `avformat_open_input`, so a stream that parks during the
    // probe/open phase is cancellable. With the token armed from the start, the
    // first read inside open aborts with `AVERROR_EXIT` (avio_read returns the
    // latched pb->error, which av_probe_input_buffer2 propagates). If the
    // install were moved after open, the mirrored callback would be null during
    // open and this read loop would hang forever instead.
    struct AlwaysInterrupt;
    impl Read for AlwaysInterrupt {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::Interrupted.into())
        }
    }

    let result = format::input_from_stream_with_interrupt(
        StreamIo::from_read(AlwaysInterrupt).unwrap(),
        None,
        None,
        || true, // armed before open
    );
    match result {
        Err(Error::Exit) => {}
        Err(e) => panic!("expected Error::Exit from the aborted open, got {:?}", e),
        Ok(_) => panic!("open must not succeed on an always-interrupted stream"),
    }
}

#[test]
fn cancel_is_honored_over_a_stream_that_keeps_returning_data() {
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Direct test of the top-of-loop interrupt poll: a stream that ALWAYS
    // returns data and NEVER returns `Interrupted`. Pre-fix the only poll was
    // inside the `Interrupted` arm, so a cancel over such a stream was silently
    // ignored and the demux ran to natural EOF. The top-of-loop poll (mirroring
    // retry_transfer_wrapper) must abort it at the next buffer refill — and the
    // stream still has data left, so the abort is `AVERROR_EXIT`, not EOF.
    struct Feed {
        inner: Cursor<Vec<u8>>,
    }
    impl Read for Feed {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }
    }
    impl Seek for Feed {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    let token = Arc::new(AtomicBool::new(false));
    let cb_token = Arc::clone(&token);
    // Far larger than the 32 KiB AVIOContext buffer, so data is still available
    // when the cancel fires: the abort can only be the interrupt, not EOF.
    let reader = Feed {
        inner: Cursor::new(tiny_wav(2_000_000)),
    };
    let mut input = format::input_from_stream_with_interrupt(
        StreamIo::from_read_seek(reader).unwrap(),
        None,
        None,
        move || cb_token.load(Ordering::SeqCst),
    )
    .expect("open with the token unfired");

    // Fire the cancel, then keep reading: some packets are still served from
    // the AVIOContext buffer, but the next refill's top-of-loop poll aborts.
    token.store(true, Ordering::SeqCst);
    let err = loop {
        match ffmpeg_next::Packet::empty().read(&mut input) {
            Ok(()) => continue,
            Err(e) => break e,
        }
    };
    assert_eq!(
        err,
        Error::Exit,
        "a cancel must abort a data-returning stream (not run to EOF)"
    );
}

#[test]
fn level_triggered_cancel_aborts_parked_read_and_seek_resumes() {
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;

    // Models a conforming network adapter parked in a blocking read when a
    // LEVEL-triggered cancel fires: every read at/past `gate` observes the
    // cancel and returns `ErrorKind::Interrupted` — and keeps returning it,
    // because the party that re-arms the token cannot run while the demux
    // thread is stuck inside the read callback. `fire_on_gate` couples the
    // "parked" moment to the cancel deterministically (single-threaded test).
    //
    // Pre-fix, the read callback retried `Interrupted` unconditionally and
    // spun here forever; now it polls the format context's interrupt callback
    // (FFmpeg's `retry_transfer_wrapper` convention) and aborts with
    // `AVERROR_EXIT`.
    struct Gated {
        inner: Cursor<Vec<u8>>,
        gate: Arc<AtomicU64>,
        token: Arc<AtomicBool>,
    }
    impl Read for Gated {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.inner.position() >= self.gate.load(Ordering::SeqCst) {
                // Parked read + the cancel arriving while parked.
                self.token.store(true, Ordering::SeqCst);
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            self.inner.read(buf)
        }
    }
    impl Seek for Gated {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    let gate = Arc::new(AtomicU64::new(u64::MAX));
    let token = Arc::new(AtomicBool::new(false));
    let reader = Gated {
        inner: Cursor::new(tiny_wav(200_000)),
        gate: Arc::clone(&gate),
        token: Arc::clone(&token),
    };
    let cb_token = Arc::clone(&token);
    let mut input = format::input_from_stream_with_interrupt(
        StreamIo::from_read_seek(reader).unwrap(),
        None,
        None,
        move || cb_token.load(Ordering::SeqCst),
    )
    .expect("open with the gate raised");

    // Drop the gate: the next protocol-level read parks + cancels. Reads may
    // still be served from the AVIOContext buffer for a while; the abort must
    // surface as a clean `Error::Exit` (NOT a hang, NOT EINTR).
    gate.store(0, Ordering::SeqCst);
    let err = loop {
        match ffmpeg_next::Packet::empty().read(&mut input) {
            Ok(()) => continue,
            Err(e) => break e,
        }
    };
    assert_eq!(
        err,
        Error::Exit,
        "parked-read cancel surfaces as AVERROR_EXIT"
    );

    // Re-arm (the canceller's job once the read returned) and resume the SAME
    // context via a seek — the deliberate post-cancel resume point. The seek
    // un-latches the aborted AVIOContext (`pb->error`/`eof_reached`), so the
    // session is fully healthy: packets flow again and the stream ends with a
    // clean EOF. Without the un-latch, `read_frame_internal` rewrites the
    // final EOF into the sticky AVERROR_EXIT and every wav read short-circuits.
    token.store(false, Ordering::SeqCst);
    gate.store(u64::MAX, Ordering::SeqCst);
    input
        .seek(0, ..0)
        .expect("post-cancel seek on the same context");
    let mut packets = 0usize;
    let end = loop {
        match ffmpeg_next::Packet::empty().read(&mut input) {
            Ok(()) => packets += 1,
            Err(e) => break e,
        }
    };
    assert!(packets > 0, "packets flow after the post-cancel seek");
    assert_eq!(
        end,
        Error::Eof,
        "clean EOF after resume (not the latched EXIT)"
    );
}

#[test]
fn url_lane_interrupt_abort_unlatches_on_seek() {
    use ffmpeg_next::ffi::{avio_read, AVERROR_EXIT};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // The URL/local-file twin of the custom-AVIO test above: FFmpeg's own
    // protocol layer (`retry_transfer_wrapper`) aborts with AVERROR_EXIT when
    // the interrupt callback fires mid-read, latching `pb->error` +
    // `eof_reached` exactly like the custom lane. A subsequent seek on the
    // same context must un-latch it, or the session ends every later
    // zero-byte read — including the true EOF — with the stale EXIT.
    // Unique per process (parallel cargo jobs / IDE + terminal share %TEMP%),
    // and removed even if an assert below panics.
    struct RemoveOnDrop(std::path::PathBuf);
    impl Drop for RemoveOnDrop {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!(
        "rust_ffmpeg_interrupt_abort_{}.wav",
        std::process::id()
    ));
    std::fs::write(&path, tiny_wav(200_000)).unwrap();
    let _cleanup = RemoveOnDrop(path.clone());

    let token = Arc::new(AtomicBool::new(false));
    let cb_token = Arc::clone(&token);
    let mut input = format::input_with_interrupt(&path, move || cb_token.load(Ordering::SeqCst))
        .expect("open local file");

    // A packet flows normally, then the cancel fires.
    ffmpeg_next::Packet::empty()
        .read(&mut input)
        .expect("pre-cancel packet");
    token.store(true, Ordering::SeqCst);
    let err = loop {
        match ffmpeg_next::Packet::empty().read(&mut input) {
            Ok(()) => continue, // drains bytes still buffered in the AVIOContext
            Err(e) => break e,
        }
    };
    assert_eq!(
        err,
        Error::Exit,
        "cancelled file read surfaces as AVERROR_EXIT"
    );

    // Force the protocol-level latch deterministically: the packet loop above
    // may surface EXIT while bytes are still buffered in the AVIOContext (or
    // from `avformat_find_stream_info`'s own interrupt check) before anything
    // is latched into `pb->error`. Drive `avio_read` until the buffered bytes
    // run out and the refill hits `retry_transfer_wrapper`'s interrupt check
    // (avio.c), which latches `AVERROR_EXIT` into `pb->error`.
    unsafe {
        let pb = (*input.as_mut_ptr()).pb;
        assert!(!pb.is_null());
        let mut buf = [0u8; 4096];
        while avio_read(pb, buf.as_mut_ptr(), buf.len() as _) > 0 {}
        assert_eq!((*pb).error, AVERROR_EXIT, "abort latched into pb->error");
    }

    // Re-arm + seek: the latch clears and the session plays to a clean EOF.
    token.store(false, Ordering::SeqCst);
    input
        .seek(0, ..0)
        .expect("post-cancel seek on the same context");
    unsafe {
        let pb = (*input.as_mut_ptr()).pb;
        assert_eq!((*pb).error, 0, "seek un-latched the aborted AVIOContext");
    }
    let mut packets = 0usize;
    let end = loop {
        match ffmpeg_next::Packet::empty().read(&mut input) {
            Ok(()) => packets += 1,
            Err(e) => break e,
        }
    };
    assert!(packets > 0, "packets flow after the post-cancel seek");
    assert_eq!(
        end,
        Error::Eof,
        "clean EOF after resume (not the latched EXIT)"
    );

    // Drop the context (closing the file) before `_cleanup` removes it.
    drop(input);
}

#[test]
fn read_buffer_handed_to_the_stream_is_initialized_and_readable() {
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    // FFmpeg hands the read callback buffers that may be freshly allocated
    // (probe buffers, internal reallocations) or hold stale bytes from
    // earlier fills; the wrapper stages the stream's read through its own
    // initialized scratch, so a safe `Read` impl may freely READ from the
    // slice it is given (uninit memory behind a `&mut [u8]` would be UB —
    // under Miri/ASan this test would catch a regression). The first scratch
    // is zero-filled; later calls may see stale bytes from earlier reads,
    // which the `Read` contract allows. A panic would unwind across the C
    // callback boundary, so violations are recorded and asserted afterwards.
    struct TouchAll {
        inner: Cursor<Vec<u8>>,
        checksum: Arc<AtomicU64>,
    }
    impl Read for TouchAll {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            // Read every byte of the handed slice (legal for a safe impl).
            let sum: u64 = buf.iter().map(|&b| b as u64).sum();
            self.checksum.fetch_add(sum, Ordering::SeqCst);
            self.inner.read(buf)
        }
    }
    impl Seek for TouchAll {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    let checksum = Arc::new(AtomicU64::new(0));
    // Large enough that the AVIOContext buffer is refilled (and wrapped)
    // several times after probing, with every payload byte non-zero.
    let reader = TouchAll {
        inner: Cursor::new(tiny_wav(200_000)),
        checksum: Arc::clone(&checksum),
    };
    let mut input =
        format::input_from_stream(StreamIo::from_read_seek(reader).unwrap(), None, None).unwrap();
    assert!(input.packets().count() > 0);
    // The reads really touched the handed slices (later fills see stale
    // non-zero wav bytes in the reused scratch, so the sum is non-zero).
    assert!(checksum.load(Ordering::SeqCst) > 0);
}

#[test]
fn failed_refill_does_not_clobber_the_buffered_window() {
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;

    // Regression test for the abort-resume corruption: the whole file prefix
    // sits in ONE AVIOContext buffer window (a single partial read below the
    // 32 KiB buffer size), the NEXT refill fails (parked read + cancel →
    // AVERROR_EXIT), and the post-cancel resume seeks BACK into the window —
    // `avio_seek`'s in-buffer fast path serves the buffered bytes WITHOUT
    // rereading. `fill_buffer`'s contract is that a failed refill leaves that
    // window intact; the old implementation pre-zeroed FFmpeg's `buf` (which
    // IS `s->buffer` in the wrap case) before blocking, so the resumed demux
    // read silently served zeroes. Byte-exact packet payloads catch that.
    struct Gated {
        inner: Cursor<Vec<u8>>,
        gate: Arc<AtomicU64>,
        token: Arc<AtomicBool>,
    }
    impl Read for Gated {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let pos = self.inner.position();
            let gate = self.gate.load(Ordering::SeqCst);
            if pos >= gate {
                self.token.store(true, Ordering::SeqCst);
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            // Serve only up to the gate (a partial read), so the buffered
            // window ends exactly at the gate.
            let n = buf.len().min((gate - pos) as usize);
            self.inner.read(&mut buf[..n])
        }
    }
    impl Seek for Gated {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    let wav = tiny_wav(60_000);
    let gate = Arc::new(AtomicU64::new(20_000)); // < the 32 KiB buffer: one window
    let token = Arc::new(AtomicBool::new(false));
    let reader = Gated {
        inner: Cursor::new(wav.clone()),
        gate: Arc::clone(&gate),
        token: Arc::clone(&token),
    };
    let cb_token = Arc::clone(&token);
    let mut input = format::input_from_stream_with_interrupt(
        StreamIo::from_read_seek(reader).unwrap(),
        None,
        None,
        move || cb_token.load(Ordering::SeqCst),
    )
    .expect("open within the available prefix");

    // Drain to the failing refill at byte 20 000.
    let err = loop {
        match ffmpeg_next::Packet::empty().read(&mut input) {
            Ok(()) => continue,
            Err(e) => break e,
        }
    };
    assert_eq!(err, Error::Exit, "parked refill aborts with AVERROR_EXIT");

    // Re-arm + resume with a seek back into the buffered window.
    token.store(false, Ordering::SeqCst);
    gate.store(u64::MAX, Ordering::SeqCst);
    input.seek(0, ..0).expect("post-cancel seek");

    // PCM packets are raw payload slices of the file: compare byte-exact
    // against the source. The stream index into the file: wav data starts at
    // byte 44 and pcm packets are sequential from there.
    let mut file_off = 44usize;
    let mut checked = 0usize;
    let mut pkt = ffmpeg_next::Packet::empty();
    while pkt.read(&mut input).is_ok() {
        let data = pkt.data().expect("pcm packet payload");
        assert_eq!(
            data,
            &wav[file_off..file_off + data.len()],
            "post-resume packet at file offset {file_off} must be served verbatim \
             (zeroes here = the failed refill clobbered the buffered window)",
        );
        file_off += data.len();
        checked += 1;
    }
    assert!(checked > 0, "post-resume packets flowed");
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
