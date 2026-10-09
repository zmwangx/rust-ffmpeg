//! `SphericalMapping::from_side_data` / `Stereo3D::from_side_data` are safe
//! functions handed caller-chosen bytes, and both structs contain fields the
//! bindings model as Rust enums over 4-byte C enums (`AVSphericalProjection`,
//! `AVStereo3DType`, `AVStereo3DView`, `AVStereo3DPrimaryEye`). An
//! out-of-range discriminant in one of those is not a value the language
//! permits to exist, so casting bytes that hold one onto the struct is
//! undefined behaviour before any accessor runs.
//!
//! Checking size and alignment is therefore not enough. These tests pin the
//! third check: every enum-typed field is screened from its raw `i32` first,
//! and a payload that fails is refused rather than transmuted.
//!
//! Buffers are built inside a `#[repr(C, align(4))]` array so the alignment arm
//! is exercised on purpose (`&buf[1..]`) rather than by luck of the allocator.
//!
//! `AVStereo3D` grew `view` in FFmpeg 4.0 and `primary_eye` / `baseline` in
//! 7.1, so everything touching those fields is gated to match.

use std::mem::{offset_of, size_of};

#[cfg(feature = "ffmpeg_4_0")]
use ffmpeg_next::ffi::AVStereo3DView;
use ffmpeg_next::ffi::{
    AV_STEREO3D_FLAG_INVERT, AVSphericalMapping, AVSphericalProjection, AVStereo3D, AVStereo3DType,
};
use ffmpeg_next::util::spherical::{Projection, SphericalMapping};
#[cfg(feature = "ffmpeg_4_0")]
use ffmpeg_next::util::stereo3d::View;
use ffmpeg_next::util::stereo3d::{Stereo3D, Type};

/// A scratch payload aligned for both structs (each is `align(4)`), so a slice
/// starting at index 0 is aligned and one starting at index 1 is not.
#[repr(C, align(4))]
struct Aligned([u8; 64]);

impl Aligned {
    fn zeroed() -> Self {
        Self([0u8; 64])
    }

    fn put_i32(&mut self, offset: usize, value: i32) {
        self.0[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
    }

    fn put_u32(&mut self, offset: usize, value: u32) {
        self.0[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
    }

    fn payload(&self, len: usize) -> &[u8] {
        &self.0[..len]
    }

    /// The same length of bytes, one byte in — an odd address, so never aligned
    /// for a 4-aligned struct.
    fn misaligned(&self, len: usize) -> &[u8] {
        &self.0[1..1 + len]
    }
}

// ─────────────────────────────────────────────────────────────────────
// AVSphericalMapping
// ─────────────────────────────────────────────────────────────────────

/// Every `Projection` this build's libavutil defines. Their discriminants are
/// `0..len`, contiguous, which is what makes `len` the first undefined one.
fn defined_projections() -> Vec<Projection> {
    #[allow(unused_mut)]
    let mut all = vec![
        Projection::Equirectangular,
        Projection::Cubemap,
        Projection::EquirectangularTile,
    ];
    #[cfg(feature = "ffmpeg_7_1")]
    all.extend([
        Projection::HalfEquirectangular,
        Projection::Rectilinear,
        Projection::Fisheye,
    ]);
    #[cfg(feature = "ffmpeg_8_0")]
    all.push(Projection::ParametricImmersive);
    all
}

/// 0.32 fixed point for `eighths / 8`, exact in an `f64`.
fn eighths(eighths: u32) -> u32 {
    eighths << 29
}

/// Every field set, the four bounds to four different values, so a test can
/// tell them apart.
fn spherical_payload(projection: i32) -> Aligned {
    let mut buf = Aligned::zeroed();
    buf.put_i32(offset_of!(AVSphericalMapping, projection), projection);
    buf.put_i32(offset_of!(AVSphericalMapping, yaw), 90 << 16);
    buf.put_i32(offset_of!(AVSphericalMapping, pitch), -45 << 16);
    buf.put_i32(offset_of!(AVSphericalMapping, roll), 30 << 15);
    buf.put_u32(offset_of!(AVSphericalMapping, bound_left), eighths(1));
    buf.put_u32(offset_of!(AVSphericalMapping, bound_top), eighths(2));
    buf.put_u32(offset_of!(AVSphericalMapping, bound_right), eighths(3));
    buf.put_u32(offset_of!(AVSphericalMapping, bound_bottom), eighths(4));
    buf.put_u32(offset_of!(AVSphericalMapping, padding), 7);
    buf
}

#[test]
fn a_well_formed_spherical_payload_is_borrowed_and_read() {
    let buf = spherical_payload(AVSphericalProjection::AV_SPHERICAL_EQUIRECTANGULAR_TILE as i32);
    let m = SphericalMapping::from_side_data(buf.payload(size_of::<AVSphericalMapping>()))
        .expect("a defined projection is a usable payload");

    assert_eq!(m.projection(), Projection::EquirectangularTile);
    assert_eq!(m.yaw(), 90.0);
    assert_eq!(m.pitch(), -45.0);
    assert_eq!(m.roll(), 15.0);
    assert_eq!(m.padding(), 7);

    // Each bound comes from its own field, unflipped: `right` and `bottom` are
    // insets from the right and bottom edges, not the tile's far coordinates.
    let bounds = m.bounds();
    assert_eq!(bounds.left, 0.125);
    assert_eq!(bounds.top, 0.25);
    assert_eq!(bounds.right, 0.375);
    assert_eq!(bounds.bottom, 0.5);

    let debug = format!("{m:?}");
    assert!(
        debug.contains("EquirectangularTile") && debug.contains("yaw: 90.0"),
        "Debug shows decoded values, not raw fixed point: {debug}"
    );
}

#[test]
fn every_defined_spherical_projection_is_admitted() {
    for expected in defined_projections() {
        let raw = AVSphericalProjection::from(expected) as i32;
        assert_eq!(Projection::from_raw(raw), Some(expected), "from_raw({raw})");

        let buf = spherical_payload(raw);
        let m = SphericalMapping::from_side_data(buf.payload(size_of::<AVSphericalMapping>()))
            .unwrap_or_else(|| panic!("{expected:?} ({raw}) is a value libavutil defines"));
        assert_eq!(m.projection(), expected);
    }
}

#[test]
fn an_undefined_spherical_projection_is_refused_not_transmuted() {
    let first_undefined = defined_projections().len() as i32;
    for raw in [-1, first_undefined, 99, i32::MIN, i32::MAX] {
        assert_eq!(
            Projection::from_raw(raw),
            None,
            "from_raw({raw}) must not name a projection"
        );

        let buf = spherical_payload(raw);
        assert!(
            SphericalMapping::from_side_data(buf.payload(size_of::<AVSphericalMapping>()))
                .is_none(),
            "projection {raw} is not an AVSphericalProjection; borrowing the payload would be UB",
        );
    }
}

#[test]
fn a_short_or_misaligned_spherical_payload_is_refused() {
    let buf = spherical_payload(AVSphericalProjection::AV_SPHERICAL_CUBEMAP as i32);
    let size = size_of::<AVSphericalMapping>();

    assert!(
        SphericalMapping::from_side_data(buf.payload(size - 1)).is_none(),
        "one byte short"
    );
    assert!(SphericalMapping::from_side_data(&[]).is_none(), "empty");
    assert!(
        SphericalMapping::from_side_data(buf.misaligned(size)).is_none(),
        "odd address"
    );
}

// ─────────────────────────────────────────────────────────────────────
// AVStereo3D
// ─────────────────────────────────────────────────────────────────────

fn defined_types() -> Vec<Type> {
    #[allow(unused_mut)]
    let mut all = vec![
        Type::TwoD,
        Type::SideBySide,
        Type::TopBottom,
        Type::FrameSequence,
        Type::Checkerboard,
        Type::SideBySideQuincunx,
        Type::Lines,
        Type::Columns,
    ];
    #[cfg(feature = "ffmpeg_7_1")]
    all.push(Type::Unspecified);
    all
}

#[cfg(feature = "ffmpeg_4_0")]
fn defined_views() -> Vec<View> {
    #[allow(unused_mut)]
    let mut all = vec![View::Packed, View::Left, View::Right];
    #[cfg(feature = "ffmpeg_7_1")]
    all.push(View::Unspecified);
    all
}

#[cfg(feature = "ffmpeg_7_1")]
fn defined_eyes() -> Vec<ffmpeg_next::util::stereo3d::PrimaryEye> {
    use ffmpeg_next::util::stereo3d::PrimaryEye;
    vec![PrimaryEye::None, PrimaryEye::Left, PrimaryEye::Right]
}

/// A payload with the given packing type. Every other enum-typed field is
/// zero — a defined discriminant in each (`PACKED`, `NONE`) — so a test can
/// poison exactly one of them by overwriting it.
fn stereo_payload(type_: i32) -> Aligned {
    let mut buf = Aligned::zeroed();
    buf.put_i32(offset_of!(AVStereo3D, type_), type_);
    buf.put_i32(offset_of!(AVStereo3D, flags), AV_STEREO3D_FLAG_INVERT);
    #[cfg(feature = "ffmpeg_7_1")]
    buf.put_u32(offset_of!(AVStereo3D, baseline), 64_000);
    buf
}

#[test]
fn a_well_formed_stereo3d_payload_is_borrowed_and_read() {
    let buf = stereo_payload(AVStereo3DType::AV_STEREO3D_SIDEBYSIDE as i32);
    let s = Stereo3D::from_side_data(buf.payload(size_of::<AVStereo3D>()))
        .expect("defined discriminants make a usable payload");

    assert_eq!(s.kind(), Type::SideBySide);
    assert!(s.inverted());
    #[cfg(feature = "ffmpeg_4_0")]
    assert_eq!(s.view(), View::Packed);
    #[cfg(feature = "ffmpeg_7_1")]
    {
        assert_eq!(
            s.primary_eye(),
            ffmpeg_next::util::stereo3d::PrimaryEye::None
        );
        assert_eq!(s.baseline(), 64_000);
    }

    let debug = format!("{s:?}");
    assert!(
        debug.contains("SideBySide") && debug.contains("inverted: true"),
        "Debug shows decoded values: {debug}"
    );
}

#[test]
fn every_defined_stereo3d_discriminant_is_admitted() {
    let size = size_of::<AVStereo3D>();

    for expected in defined_types() {
        let raw = AVStereo3DType::from(expected) as i32;
        assert_eq!(Type::from_raw(raw), Some(expected), "Type::from_raw({raw})");
        let buf = stereo_payload(raw);
        let s = Stereo3D::from_side_data(buf.payload(size))
            .unwrap_or_else(|| panic!("type {raw} is defined"));
        assert_eq!(s.kind(), expected);
    }

    #[cfg(feature = "ffmpeg_4_0")]
    for expected in defined_views() {
        let raw = AVStereo3DView::from(expected) as i32;
        assert_eq!(View::from_raw(raw), Some(expected), "View::from_raw({raw})");
        let mut buf = stereo_payload(0);
        buf.put_i32(offset_of!(AVStereo3D, view), raw);
        let s = Stereo3D::from_side_data(buf.payload(size))
            .unwrap_or_else(|| panic!("view {raw} is defined"));
        assert_eq!(s.view(), expected);
    }

    #[cfg(feature = "ffmpeg_7_1")]
    for expected in defined_eyes() {
        use ffmpeg_next::ffi::AVStereo3DPrimaryEye;
        use ffmpeg_next::util::stereo3d::PrimaryEye;
        let raw = AVStereo3DPrimaryEye::from(expected) as i32;
        assert_eq!(
            PrimaryEye::from_raw(raw),
            Some(expected),
            "PrimaryEye::from_raw({raw})"
        );
        let mut buf = stereo_payload(0);
        buf.put_i32(offset_of!(AVStereo3D, primary_eye), raw);
        let s = Stereo3D::from_side_data(buf.payload(size))
            .unwrap_or_else(|| panic!("primary_eye {raw} is defined"));
        assert_eq!(s.primary_eye(), expected);
    }
}

#[test]
fn an_undefined_stereo3d_discriminant_is_refused_in_every_enum_field() {
    let size = size_of::<AVStereo3D>();
    let bad_type = defined_types().len() as i32;

    for raw in [-1, bad_type, 99, i32::MIN, i32::MAX] {
        assert_eq!(Type::from_raw(raw), None, "Type::from_raw({raw})");
        let buf = stereo_payload(raw);
        assert!(
            Stereo3D::from_side_data(buf.payload(size)).is_none(),
            "type {raw} is not an AVStereo3DType; borrowing the payload would be UB",
        );
    }

    #[cfg(feature = "ffmpeg_4_0")]
    {
        let bad_view = defined_views().len() as i32;
        for raw in [-1, bad_view, 99, i32::MIN, i32::MAX] {
            assert_eq!(View::from_raw(raw), None, "View::from_raw({raw})");
            let mut buf = stereo_payload(0);
            buf.put_i32(offset_of!(AVStereo3D, view), raw);
            assert!(
                Stereo3D::from_side_data(buf.payload(size)).is_none(),
                "view {raw} is not an AVStereo3DView; borrowing the payload would be UB",
            );
        }
    }

    #[cfg(feature = "ffmpeg_7_1")]
    {
        use ffmpeg_next::util::stereo3d::PrimaryEye;
        let bad_eye = defined_eyes().len() as i32;
        for raw in [-1, bad_eye, 99, i32::MIN, i32::MAX] {
            assert_eq!(
                PrimaryEye::from_raw(raw),
                None,
                "PrimaryEye::from_raw({raw})"
            );
            let mut buf = stereo_payload(0);
            buf.put_i32(offset_of!(AVStereo3D, primary_eye), raw);
            assert!(
                Stereo3D::from_side_data(buf.payload(size)).is_none(),
                "primary_eye {raw} is not an AVStereo3DPrimaryEye; borrowing the payload would be UB",
            );
        }
    }
}

#[test]
fn a_short_or_misaligned_stereo3d_payload_is_refused() {
    let buf = stereo_payload(0);
    let size = size_of::<AVStereo3D>();

    assert!(
        Stereo3D::from_side_data(buf.payload(size - 1)).is_none(),
        "one byte short"
    );
    assert!(Stereo3D::from_side_data(&[]).is_none(), "empty");
    assert!(
        Stereo3D::from_side_data(buf.misaligned(size)).is_none(),
        "odd address"
    );
}
