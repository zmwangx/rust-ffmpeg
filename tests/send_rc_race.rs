//! Regression test for the unsound `Send` impls over `Rc`-holding wrappers.
//!
//! `Stream::parameters()` returns a `codec::Parameters` that shares ownership
//! of the backing `AVFormatContext` with the `format::context::Input` it came
//! from. Both types are `Send`, so the shared refcount must be atomic: this
//! test drops batches of such clones from several threads at once and then
//! checks the refcount through a weak handle. With the old non-atomic `Rc`
//! keep-alive, racing decrements lose updates and leave the count above zero
//! (context leaked) or hit zero early (double free). With `Arc` the count is
//! exactly zero every time.

extern crate ffmpeg_next as ffmpeg;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;

const THREADS: usize = 4;
const CLONES_PER_THREAD: usize = 64;

fn iterations() -> usize {
    std::env::var("SEND_RC_RACE_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1_000)
}

/// Write a minimal valid WAV file (PCM s16le, mono, 8 kHz, 0.2 s of silence).
fn make_wav_fixture() -> PathBuf {
    let data_len: u32 = 3200;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&8000u32.to_le_bytes()); // sample rate
    bytes.extend_from_slice(&16000u32.to_le_bytes()); // byte rate
    bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
    bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    bytes.resize(bytes.len() + data_len as usize, 0);

    let path = std::env::temp_dir().join(format!(
        "ffmpeg-next-send-rc-race-{}.wav",
        std::process::id()
    ));
    File::create(&path).unwrap().write_all(&bytes).unwrap();
    path
}

#[test]
fn parameters_send_rc_race() {
    ffmpeg::init().unwrap();
    let path = make_wav_fixture();

    for i in 1..=iterations() {
        let input = ffmpeg::format::input(&path).unwrap();
        // Observe the shared keep-alive refcount from outside.
        let weak = Arc::downgrade(&unsafe { input.destructor() });

        let barrier = Arc::new(Barrier::new(THREADS + 1));
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let batch: Vec<_> = (0..CLONES_PER_THREAD)
                    .map(|_| input.streams().next().unwrap().parameters())
                    .collect();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    drop(batch); // tight loop of refcount decrements
                })
            })
            .collect();

        barrier.wait();
        drop(input);
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(
            weak.strong_count(),
            0,
            "keep-alive refcount corrupted by unsynchronized drops (iteration {})",
            i
        );
    }

    std::fs::remove_file(&path).ok();
}

#[test]
fn wrappers_are_send() {
    fn assert_send<T: Send>() {}

    assert_send::<ffmpeg::codec::Parameters>();
    assert_send::<ffmpeg::codec::Context>();
    assert_send::<ffmpeg::format::context::Input>();
    assert_send::<ffmpeg::format::context::Output>();
    assert_send::<ffmpeg::format::context::Context>();
}
