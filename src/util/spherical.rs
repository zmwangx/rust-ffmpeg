use std::fmt;
use std::mem;

use crate::ffi::AVSphericalProjection::*;
use crate::ffi::*;

/// Projection of the video surface(s) on a sphere — the safe twin of
/// `AVSphericalProjection`.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Projection {
    /// A sphere mapped onto a flat surface with an equirectangular projection.
    Equirectangular,
    /// Six cube faces on a 3x2 layout. Front, left, right and back are stored
    /// upright; the up face's top edge points forward and the down face's top
    /// edge points to the back.
    Cubemap,
    /// A PORTION of an equirectangular sphere; the bounding fields say where.
    EquirectangularTile,
    /// A 180-degree equirectangular projection.
    #[cfg(feature = "ffmpeg_7_1")]
    HalfEquirectangular,
    /// A flat, rectangular 2D surface.
    #[cfg(feature = "ffmpeg_7_1")]
    Rectilinear,
    /// Fisheye (Apple).
    #[cfg(feature = "ffmpeg_7_1")]
    Fisheye,
    /// Parametric immersive (Apple).
    #[cfg(feature = "ffmpeg_8_0")]
    ParametricImmersive,
}

impl From<AVSphericalProjection> for Projection {
    #[inline(always)]
    fn from(value: AVSphericalProjection) -> Projection {
        match value {
            AV_SPHERICAL_EQUIRECTANGULAR => Projection::Equirectangular,
            AV_SPHERICAL_CUBEMAP => Projection::Cubemap,
            AV_SPHERICAL_EQUIRECTANGULAR_TILE => Projection::EquirectangularTile,
            #[cfg(feature = "ffmpeg_7_1")]
            AV_SPHERICAL_HALF_EQUIRECTANGULAR => Projection::HalfEquirectangular,
            #[cfg(feature = "ffmpeg_7_1")]
            AV_SPHERICAL_RECTILINEAR => Projection::Rectilinear,
            #[cfg(feature = "ffmpeg_7_1")]
            AV_SPHERICAL_FISHEYE => Projection::Fisheye,
            #[cfg(feature = "ffmpeg_8_0")]
            AV_SPHERICAL_PARAMETRIC_IMMERSIVE => Projection::ParametricImmersive,

            #[cfg(feature = "non-exhaustive-enums")]
            _ => unimplemented!(),
        }
    }
}

impl From<Projection> for AVSphericalProjection {
    #[inline(always)]
    fn from(value: Projection) -> AVSphericalProjection {
        match value {
            Projection::Equirectangular => AV_SPHERICAL_EQUIRECTANGULAR,
            Projection::Cubemap => AV_SPHERICAL_CUBEMAP,
            Projection::EquirectangularTile => AV_SPHERICAL_EQUIRECTANGULAR_TILE,
            #[cfg(feature = "ffmpeg_7_1")]
            Projection::HalfEquirectangular => AV_SPHERICAL_HALF_EQUIRECTANGULAR,
            #[cfg(feature = "ffmpeg_7_1")]
            Projection::Rectilinear => AV_SPHERICAL_RECTILINEAR,
            #[cfg(feature = "ffmpeg_7_1")]
            Projection::Fisheye => AV_SPHERICAL_FISHEYE,
            #[cfg(feature = "ffmpeg_8_0")]
            Projection::ParametricImmersive => AV_SPHERICAL_PARAMETRIC_IMMERSIVE,
        }
    }
}

impl Projection {
    /// The safe twin of a raw `AVSphericalProjection` discriminant; `None` for
    /// a value this build's libavutil does not define.
    ///
    /// The bindings model `AVSphericalProjection` as a Rust enum over a 4-byte
    /// C enum (bindgen picks `i32` or `u32` for its repr, depending on the
    /// target), so a value holding any other discriminant does not exist as
    /// far as the language is concerned: producing one — by casting bytes onto
    /// the struct that contains it — is undefined behaviour on the spot,
    /// before any accessor reads it. Every path from untrusted bytes to a
    /// [`SphericalMapping`] goes through here.
    pub fn from_raw(raw: i32) -> Option<Projection> {
        // Bound as `const`s so they can be used as match patterns.
        const EQUIRECTANGULAR: i32 = AV_SPHERICAL_EQUIRECTANGULAR as i32;
        const CUBEMAP: i32 = AV_SPHERICAL_CUBEMAP as i32;
        const EQUIRECTANGULAR_TILE: i32 = AV_SPHERICAL_EQUIRECTANGULAR_TILE as i32;
        #[cfg(feature = "ffmpeg_7_1")]
        const HALF_EQUIRECTANGULAR: i32 = AV_SPHERICAL_HALF_EQUIRECTANGULAR as i32;
        #[cfg(feature = "ffmpeg_7_1")]
        const RECTILINEAR: i32 = AV_SPHERICAL_RECTILINEAR as i32;
        #[cfg(feature = "ffmpeg_7_1")]
        const FISHEYE: i32 = AV_SPHERICAL_FISHEYE as i32;
        #[cfg(feature = "ffmpeg_8_0")]
        const PARAMETRIC_IMMERSIVE: i32 = AV_SPHERICAL_PARAMETRIC_IMMERSIVE as i32;

        match raw {
            EQUIRECTANGULAR => Some(Projection::Equirectangular),
            CUBEMAP => Some(Projection::Cubemap),
            EQUIRECTANGULAR_TILE => Some(Projection::EquirectangularTile),
            #[cfg(feature = "ffmpeg_7_1")]
            HALF_EQUIRECTANGULAR => Some(Projection::HalfEquirectangular),
            #[cfg(feature = "ffmpeg_7_1")]
            RECTILINEAR => Some(Projection::Rectilinear),
            #[cfg(feature = "ffmpeg_7_1")]
            FISHEYE => Some(Projection::Fisheye),
            #[cfg(feature = "ffmpeg_8_0")]
            PARAMETRIC_IMMERSIVE => Some(Projection::ParametricImmersive),
            _ => None,
        }
    }
}

/// Where a [`Projection::EquirectangularTile`] tile sits on the full
/// equirectangular surface it was cut from.
///
/// Each field is an inset, not a coordinate: the distance from one edge of
/// the full surface to the same edge of the tile, as a fraction of the full
/// surface's width (`left`, `right`) or height (`top`, `bottom`). The tile
/// therefore spans `left..1.0 - right` horizontally and `top..1.0 - bottom`
/// vertically, and the full surface measures
/// `tile_width / (1.0 - left - right)` by
/// `tile_height / (1.0 - top - bottom)` pixels.
///
/// Meaningful only for [`Projection::EquirectangularTile`]; ignore them for
/// every other projection.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Bounds {
    /// Distance from the full surface's left edge to the tile's left edge.
    pub left: f64,
    /// Distance from the full surface's top edge to the tile's top edge.
    pub top: f64,
    /// Distance from the full surface's right edge to the tile's right edge.
    pub right: f64,
    /// Distance from the full surface's bottom edge to the tile's bottom edge.
    pub bottom: f64,
}

/// A borrowed `AVSphericalMapping` — the payload of
/// `AV_PKT_DATA_SPHERICAL` / `AV_FRAME_DATA_SPHERICAL` side data.
///
/// The accessors convert the two fixed-point encodings the struct uses (16.16
/// for the angles, 0.32 for the bounds).
///
/// [`yaw`](Self::yaw), [`pitch`](Self::pitch) and [`roll`](Self::roll) are the
/// initial orientation: rotations applied to the sphere after the frame is
/// mapped onto it, around a viewer who stays still. They always compose in
/// the same order — yaw, then pitch, then roll — which, in OpenGL axes (x
/// right, y up, z the forward vector pointing out of the screen), is the
/// rotation `R = Ry(yaw) · Rx(pitch) · Rz(roll)`.
#[derive(Clone, Copy)]
pub struct SphericalMapping<'a> {
    mapping: &'a AVSphericalMapping,
}

impl<'a> SphericalMapping<'a> {
    /// # Safety
    ///
    /// `ptr` must point at a live, correctly aligned `AVSphericalMapping` that
    /// outlives `'a`, is not written to during `'a`, and whose `projection`
    /// field holds a discriminant this build's `AVSphericalProjection`
    /// defines — see [`Projection::from_raw`].
    ///
    /// A pointer libavutil produced meets the last condition only when the
    /// runtime libavutil is not newer than the headers these bindings were
    /// generated from: a newer, still ABI-compatible release can write a
    /// projection this build has no variant for.
    /// [`from_side_data`](Self::from_side_data) screens for that and is the
    /// safe way in.
    pub unsafe fn wrap(ptr: *const AVSphericalMapping) -> Self {
        SphericalMapping {
            mapping: unsafe { &*ptr },
        }
    }

    /// Borrow a side-data payload as an `AVSphericalMapping`.
    ///
    /// `None` when the buffer cannot be one:
    ///
    /// * it is shorter than the struct;
    /// * it is not aligned for the struct;
    /// * its `projection` field holds a value this build's
    ///   `AVSphericalProjection` does not define, which is screened from the
    ///   raw `i32` here because the cast itself would already be undefined
    ///   behaviour.
    ///
    /// libavutil allocates the payload with `av_spherical_alloc`, so a genuine
    /// side-data buffer is long enough and aligned. Its projection is one this
    /// build defines unless the runtime libavutil is newer than these bindings
    /// and used a projection they lack, in which case `None` is the only sound
    /// answer.
    pub fn from_side_data(data: &'a [u8]) -> Option<Self> {
        if data.len() < mem::size_of::<AVSphericalMapping>() {
            return None;
        }
        let ptr = data.as_ptr() as *const AVSphericalMapping;
        if !ptr.is_aligned() {
            return None;
        }
        // SAFETY: `data` is long enough for the whole struct and aligned for
        // it, so `(*ptr).projection` is a readable, aligned `i32`. `&raw const`
        // takes its address without loading it at the declared enum type —
        // that load is exactly the undefined behaviour being screened for.
        let projection = unsafe { (&raw const (*ptr).projection).cast::<i32>().read() };
        Projection::from_raw(projection)?;
        // SAFETY: size, alignment and the one enum-typed field are checked
        // above, so `wrap`'s contract holds, and the borrow of `data` is
        // carried into the returned lifetime.
        Some(unsafe { Self::wrap(ptr) })
    }

    pub fn as_ptr(&self) -> *const AVSphericalMapping {
        self.mapping
    }

    pub fn projection(&self) -> Projection {
        Projection::from(self.mapping.projection)
    }

    /// Rotation around the up vector, degrees in `[-180, 180]`; applied first.
    /// Positive moves the part of the sphere in front of the viewer toward
    /// their right.
    pub fn yaw(&self) -> f64 {
        fixed_16_16(self.mapping.yaw)
    }

    /// Rotation around the right vector, degrees in `[-90, 90]`; applied after
    /// yaw. Positive moves the part of the sphere in front of the viewer
    /// upwards.
    pub fn pitch(&self) -> f64 {
        fixed_16_16(self.mapping.pitch)
    }

    /// Rotation around the forward vector, degrees in `[-180, 180]`; applied
    /// last. Positive tilts the part of the sphere in front of the viewer to
    /// the viewer's right.
    pub fn roll(&self) -> f64 {
        fixed_16_16(self.mapping.roll)
    }

    /// Where the tile sits on the full surface, as insets from each of its
    /// edges — see [`Bounds`].
    pub fn bounds(&self) -> Bounds {
        Bounds {
            left: fixed_0_32(self.mapping.bound_left),
            top: fixed_0_32(self.mapping.bound_top),
            right: fixed_0_32(self.mapping.bound_right),
            bottom: fixed_0_32(self.mapping.bound_bottom),
        }
    }

    /// Pixels of padding at the edge of each cube face. Meaningful only for
    /// [`Projection::Cubemap`].
    pub fn padding(&self) -> u32 {
        self.mapping.padding
    }
}

impl fmt::Debug for SphericalMapping<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SphericalMapping")
            .field("projection", &self.projection())
            .field("yaw", &self.yaw())
            .field("pitch", &self.pitch())
            .field("roll", &self.roll())
            .field("bounds", &self.bounds())
            .field("padding", &self.padding())
            .finish()
    }
}

/// 16.16 signed fixed point, as the angles are exported.
#[inline(always)]
fn fixed_16_16(value: i32) -> f64 {
    f64::from(value) / 65536.0
}

/// 0.32 unsigned fixed point, as the bounds are exported.
#[inline(always)]
fn fixed_0_32(value: u32) -> f64 {
    f64::from(value) / 4_294_967_296.0
}
