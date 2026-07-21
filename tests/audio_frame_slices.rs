extern crate ffmpeg_next as ffmpeg;

use crate::ffmpeg::{
    ChannelLayout,
    format::{Sample, sample::Type},
    frame,
};

const SAMPLES: usize = 1024;

#[test]
fn packed_plane_lengths() {
    let frame = frame::Audio::new(Sample::F32(Type::Packed), SAMPLES, ChannelLayout::STEREO);

    assert_eq!(frame.planes(), 1);
    assert_eq!(frame.plane::<f32>(0).len(), SAMPLES * 2);
    assert_eq!(frame.plane::<(f32, f32)>(0).len(), SAMPLES);
    assert!(frame.data(0).len() >= SAMPLES * 2 * 4);
}

#[test]
fn planar_plane_lengths() {
    let frame = frame::Audio::new(Sample::F32(Type::Planar), SAMPLES, ChannelLayout::STEREO);

    assert_eq!(frame.planes(), 2);
    assert_eq!(frame.plane::<f32>(0).len(), SAMPLES);
    assert_eq!(frame.plane::<f32>(1).len(), SAMPLES);

    // linesize[1] is never set for audio; data(1) must use linesize[0].
    assert!(frame.data(0).len() >= SAMPLES * 4);
    assert_eq!(frame.data(1).len(), frame.data(0).len());
}

#[test]
fn planar_more_than_eight_channels() {
    let mut frame = frame::Audio::new(
        Sample::F32(Type::Planar),
        SAMPLES,
        ChannelLayout::HEXADECAGONAL,
    );

    assert_eq!(frame.planes(), 16);

    // Planes past AVFrame.data's 8 slots live in extended_data.
    for i in 0..16 {
        frame.plane_mut::<f32>(i).fill(i as f32);
        assert!(frame.data(i).len() >= SAMPLES * 4);
    }

    for i in 0..16 {
        let plane = frame.plane::<f32>(i);
        assert_eq!(plane.len(), SAMPLES);
        assert!(plane.iter().all(|&s| s == i as f32));
    }
}

#[test]
fn packed_roundtrip_through_tuples() {
    let mut frame = frame::Audio::new(Sample::I16(Type::Packed), SAMPLES, ChannelLayout::STEREO);

    for (i, sample) in frame.plane_mut::<(i16, i16)>(0).iter_mut().enumerate() {
        *sample = (i as i16, -(i as i16));
    }

    let scalars = frame.plane::<i16>(0);
    assert_eq!(scalars.len(), SAMPLES * 2);
    assert_eq!(scalars[0], 0);
    assert_eq!(scalars[1], 0);
    assert_eq!(scalars[2], 1);
    assert_eq!(scalars[3], -1);
}
