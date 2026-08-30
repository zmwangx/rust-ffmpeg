//! `SphericalMapping::from_side_data` / `Stereo3D::from_side_data` are safe
//! functions handed caller-chosen bytes, and both structs contain `#[repr(i32)]`
//! enums (`AVSphericalProjection`, `AVStereo3DType`, `AVStereo3DView`,
//! `AVStereo3DPrimaryEye`). An out-of-range discriminant in one of those is not
//! a value the language permits to exist, so casting bytes that hold one onto
//! the struct is undefined behaviour before any accessor runs.
//!
//! Checking size and alignment is therefore not enough. These tests pin the
//! third check: every enum-typed field is screened from its raw `i32` first,
//! and a payload that fails is refused rather than transmuted.
//!
//! Buffers are built inside a `#[repr(C, align(4))]` array so the alignment arm
//! is exercised on purpose (`&buf[1..]`) rather than by luck of the allocator.

use std::mem::{offset_of, size_of};

use ffmpeg_next::ffi::{
    AVSphericalMapping, AVSphericalProjection, AVStereo3D, AVStereo3DType, AVStereo3DView,
};
use ffmpeg_next::util::spherical::{Projection, SphericalMapping};
use ffmpeg_next::util::stereo3d::{Stereo3D, Type, View};

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

fn spherical_payload(projection: i32) -> Aligned {
    let mut buf = Aligned::zeroed();
    buf.put_i32(offset_of!(AVSphericalMapping, projection), projection);
    buf.put_i32(offset_of!(AVSphericalMapping, yaw), 90 << 16);
    buf.put_i32(offset_of!(AVSphericalMapping, pitch), -45 << 16);
    buf.put_i32(offset_of!(AVSphericalMapping, roll), 0);
    buf.put_u32(
        offset_of!(AVSphericalMapping, bound_left),
        (u32::MAX / 4) + 1,
    );
    buf.put_u32(offset_of!(AVSphericalMapping, padding), 7);
    buf
}

#[test]
fn a_well_formed_spherical_payload_is_borrowed_and_read() {
    let buf = spherical_payload(AVSphericalProjection::AV_SPHERICAL_EQUIRECTANGULAR as i32);
    let m = SphericalMapping::from_side_data(buf.payload(size_of::<AVSphericalMapping>()))
        .expect("a defined projection is a usable payload");

    assert_eq!(m.projection(), Projection::Equirectangular);
    assert_eq!(m.yaw(), 90.0);
    assert_eq!(m.pitch(), -45.0);
    assert_eq!(m.bounds().left, 0.25);
    assert_eq!(m.padding(), 7);
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

/// A payload whose three enum-typed fields are set independently, so a test can
/// poison exactly one of them.
fn stereo_payload(type_: i32, view: i32, primary_eye: i32) -> Aligned {
    let mut buf = Aligned::zeroed();
    buf.put_i32(offset_of!(AVStereo3D, type_), type_);
    buf.put_i32(offset_of!(AVStereo3D, view), view);
    buf.put_i32(offset_of!(AVStereo3D, primary_eye), primary_eye);
    buf.put_i32(offset_of!(AVStereo3D, flags), 1); // AV_STEREO3D_FLAG_INVERT
    buf.put_u32(offset_of!(AVStereo3D, baseline), 64_000);
    buf
}

#[test]
fn a_well_formed_stereo3d_payload_is_borrowed_and_read() {
    let buf = stereo_payload(
        AVStereo3DType::AV_STEREO3D_SIDEBYSIDE as i32,
        AVStereo3DView::AV_STEREO3D_VIEW_PACKED as i32,
        0,
    );
    let s = Stereo3D::from_side_data(buf.payload(size_of::<AVStereo3D>()))
        .expect("defined discriminants make a usable payload");

    assert_eq!(s.kind(), Type::SideBySide);
    assert_eq!(s.view(), View::Packed);
    assert!(s.inverted());
    #[cfg(feature = "ffmpeg_7_1")]
    {
        assert_eq!(
            s.primary_eye(),
            ffmpeg_next::util::stereo3d::PrimaryEye::None
        );
        assert_eq!(s.baseline(), 64_000);
    }
}

#[test]
fn every_defined_stereo3d_discriminant_is_admitted() {
    let size = size_of::<AVStereo3D>();

    for expected in defined_types() {
        let raw = AVStereo3DType::from(expected) as i32;
        assert_eq!(Type::from_raw(raw), Some(expected), "Type::from_raw({raw})");
        let buf = stereo_payload(raw, 0, 0);
        let s = Stereo3D::from_side_data(buf.payload(size))
            .unwrap_or_else(|| panic!("type {raw} is defined"));
        assert_eq!(s.kind(), expected);
    }

    for expected in defined_views() {
        let raw = AVStereo3DView::from(expected) as i32;
        assert_eq!(View::from_raw(raw), Some(expected), "View::from_raw({raw})");
        let buf = stereo_payload(0, raw, 0);
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
        let buf = stereo_payload(0, 0, raw);
        let s = Stereo3D::from_side_data(buf.payload(size))
            .unwrap_or_else(|| panic!("primary_eye {raw} is defined"));
        assert_eq!(s.primary_eye(), expected);
    }
}

#[test]
fn an_undefined_stereo3d_discriminant_is_refused_in_every_enum_field() {
    let size = size_of::<AVStereo3D>();
    let bad_type = defined_types().len() as i32;
    let bad_view = defined_views().len() as i32;

    for raw in [-1, bad_type, 99, i32::MIN, i32::MAX] {
        assert_eq!(Type::from_raw(raw), None, "Type::from_raw({raw})");
        let buf = stereo_payload(raw, 0, 0);
        assert!(
            Stereo3D::from_side_data(buf.payload(size)).is_none(),
            "type {raw} is not an AVStereo3DType; borrowing the payload would be UB",
        );
    }

    for raw in [-1, bad_view, 99, i32::MIN, i32::MAX] {
        assert_eq!(View::from_raw(raw), None, "View::from_raw({raw})");
        let buf = stereo_payload(0, raw, 0);
        assert!(
            Stereo3D::from_side_data(buf.payload(size)).is_none(),
            "view {raw} is not an AVStereo3DView; borrowing the payload would be UB",
        );
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
            let buf = stereo_payload(0, 0, raw);
            assert!(
                Stereo3D::from_side_data(buf.payload(size)).is_none(),
                "primary_eye {raw} is not an AVStereo3DPrimaryEye; borrowing the payload would be UB",
            );
        }
    }
}

#[test]
fn a_short_or_misaligned_stereo3d_payload_is_refused() {
    let buf = stereo_payload(0, 0, 0);
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
