use std::marker::PhantomData;
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
    /// `AVSphericalProjection` is a `#[repr(i32)]` enum, so a value holding any
    /// other discriminant does not exist as far as the language is concerned:
    /// producing one — by casting bytes onto the struct that contains it — is
    /// undefined behaviour on the spot, before any accessor reads it. Every
    /// path from untrusted bytes to a [`SphericalMapping`] goes through here.
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

/// The four `bound_*` fields as fractions of the full surface.
///
/// Meaningful only for [`Projection::EquirectangularTile`]; ignore them for
/// every other projection.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct Bounds {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// A borrowed `AVSphericalMapping` — the payload of
/// `AV_PKT_DATA_SPHERICAL` / `AV_FRAME_DATA_SPHERICAL` side data.
///
/// The accessors convert the two fixed-point encodings the struct uses (16.16
/// for the angles, 0.32 for the bounds).
pub struct SphericalMapping<'a> {
    ptr: *const AVSphericalMapping,

    _marker: PhantomData<&'a AVSphericalMapping>,
}

impl<'a> SphericalMapping<'a> {
    /// # Safety
    ///
    /// `ptr` must point at a live, correctly aligned `AVSphericalMapping` that
    /// outlives `'a` and whose `projection` field holds a discriminant
    /// `AVSphericalProjection` defines — see [`Projection::from_raw`]. A
    /// pointer libavutil produced satisfies all of it.
    pub unsafe fn wrap(ptr: *const AVSphericalMapping) -> Self {
        SphericalMapping {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Borrow a side-data payload as an `AVSphericalMapping`.
    ///
    /// `None` when the buffer cannot be one:
    ///
    /// * it is shorter than the struct;
    /// * it is not aligned for the struct;
    /// * its `projection` field holds a value `AVSphericalProjection` does not
    ///   define, which is screened from the raw `i32` here because the cast
    ///   itself would already be undefined behaviour.
    ///
    /// libavutil allocates the payload with `av_spherical_alloc` and writes
    /// only defined values into it, so a genuine side-data buffer always
    /// passes.
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
        self.ptr
    }

    pub fn projection(&self) -> Projection {
        unsafe { Projection::from((*self.as_ptr()).projection) }
    }

    /// Rotation around the up vector, degrees. Positive moves the part of the
    /// sphere in front of the viewer toward their right.
    pub fn yaw(&self) -> f64 {
        unsafe { fixed_16_16((*self.as_ptr()).yaw) }
    }

    /// Rotation around the right vector, degrees. Positive moves the part of
    /// the sphere in front of the viewer upwards.
    pub fn pitch(&self) -> f64 {
        unsafe { fixed_16_16((*self.as_ptr()).pitch) }
    }

    /// Rotation around the forward vector, degrees. Positive tilts the part of
    /// the sphere in front of the viewer to the viewer's right.
    pub fn roll(&self) -> f64 {
        unsafe { fixed_16_16((*self.as_ptr()).roll) }
    }

    /// The bounding rectangle as fractions of the full surface.
    pub fn bounds(&self) -> Bounds {
        unsafe {
            let m = &*self.as_ptr();
            Bounds {
                left: fixed_0_32(m.bound_left),
                top: fixed_0_32(m.bound_top),
                right: fixed_0_32(m.bound_right),
                bottom: fixed_0_32(m.bound_bottom),
            }
        }
    }

    /// Pixels of padding at the edge of each cube face. Meaningful only for
    /// [`Projection::Cubemap`].
    pub fn padding(&self) -> u32 {
        unsafe { (*self.as_ptr()).padding }
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
