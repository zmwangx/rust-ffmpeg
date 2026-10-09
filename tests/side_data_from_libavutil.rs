//! Side data libavutil allocated itself — the payloads demuxers and decoders
//! hand out — read back through the crate's packet and frame `SideData`
//! handles.
//!
//! What is pinned here:
//!
//! * a genuine allocation passes `from_side_data`'s size, alignment and
//!   discriminant screening, so the screening only ever refuses foreign bytes;
//! * `SideData::data` carries the packet's / frame's lifetime rather than the
//!   handle's, so a wrapper can be borrowed straight off the temporary handle
//!   and outlive it;
//! * `Frame::new_side_data` hands out zero-filled bytes, never uninitialised
//!   ones.

use ffmpeg_next::codec::packet::{Mut, Packet};
use ffmpeg_next::ffi::{
    AV_STEREO3D_FLAG_INVERT, AVPacketSideDataType, AVSphericalProjection, AVStereo3DType,
    av_packet_add_side_data, av_spherical_alloc, av_stereo3d_create_side_data,
};
use ffmpeg_next::frame;
use ffmpeg_next::util::spherical::{Projection, SphericalMapping};
use ffmpeg_next::util::stereo3d::{Stereo3D, Type};

#[test]
fn spherical_packet_side_data_from_libavutil_is_admitted() {
    let mut packet = Packet::empty();
    unsafe {
        let mut size = 0;
        let mapping = av_spherical_alloc(&mut size);
        assert!(!mapping.is_null());
        (*mapping).projection = AVSphericalProjection::AV_SPHERICAL_CUBEMAP;
        (*mapping).yaw = 30 << 16;
        (*mapping).padding = 4;
        // On success the packet owns `mapping`.
        let ret = av_packet_add_side_data(
            packet.as_mut_ptr(),
            AVPacketSideDataType::AV_PKT_DATA_SPHERICAL,
            mapping.cast(),
            size as _,
        );
        assert_eq!(ret, 0);
    }

    // Borrowed off the temporary `SideData`, which is dropped at the end of
    // this statement: this compiles only because `data()` returns the
    // packet's lifetime.
    let m = SphericalMapping::from_side_data(packet.side_data().next().unwrap().data())
        .expect("libavutil's own allocation is an AVSphericalMapping");

    assert_eq!(m.projection(), Projection::Cubemap);
    assert_eq!(m.yaw(), 30.0);
    assert_eq!(m.padding(), 4);
}

#[test]
fn stereo3d_frame_side_data_from_libavutil_is_admitted() {
    let mut frame = frame::Video::empty();
    unsafe {
        let stereo = av_stereo3d_create_side_data(frame.as_mut_ptr());
        assert!(!stereo.is_null());
        (*stereo).type_ = AVStereo3DType::AV_STEREO3D_TOPBOTTOM;
        (*stereo).flags = AV_STEREO3D_FLAG_INVERT;
    }

    // Same as above for frames: `data()` returns the frame's lifetime.
    let s = Stereo3D::from_side_data(
        frame
            .side_data(frame::side_data::Type::Stereo3D)
            .unwrap()
            .data(),
    )
    .expect("libavutil's own allocation is an AVStereo3D");

    assert_eq!(s.kind(), Type::TopBottom);
    assert!(s.inverted());
}

#[test]
fn borrowed_wrappers_are_send_sync_and_copy() {
    fn assert_send_sync_copy<T: Send + Sync + Copy>() {}
    assert_send_sync_copy::<SphericalMapping<'static>>();
    assert_send_sync_copy::<Stereo3D<'static>>();
}

#[test]
fn new_frame_side_data_is_zero_filled() {
    let mut frame = frame::Video::empty();
    let side_data = frame
        .new_side_data(frame::side_data::Type::Spherical, 4096)
        .expect("allocation");

    assert_eq!(side_data.data().len(), 4096);
    assert!(side_data.data().iter().all(|&byte| byte == 0));
}
