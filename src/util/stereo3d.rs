use std::marker::PhantomData;
use std::mem;

use crate::ffi::AVStereo3DType::*;
use crate::ffi::*;
#[cfg(feature = "ffmpeg_7_1")]
use crate::util::rational::Rational;

/// How two views are packed within one video surface — the safe twin of
/// `AVStereo3DType`.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Type {
    /// Not stereoscopic.
    TwoD,
    /// Views are next to each other.
    SideBySide,
    /// Views are on top of each other.
    TopBottom,
    /// Views alternate temporally.
    FrameSequence,
    /// Views are packed in a per-pixel checkerboard.
    Checkerboard,
    /// Side by side, upscaled with a checkerboard pattern.
    SideBySideQuincunx,
    /// Views packed per line, as if interlaced.
    Lines,
    /// Views packed per column.
    Columns,
    /// Stereoscopic, but the packing is unspecified.
    #[cfg(feature = "ffmpeg_7_1")]
    Unspecified,
}

impl From<AVStereo3DType> for Type {
    #[inline(always)]
    fn from(value: AVStereo3DType) -> Type {
        match value {
            AV_STEREO3D_2D => Type::TwoD,
            AV_STEREO3D_SIDEBYSIDE => Type::SideBySide,
            AV_STEREO3D_TOPBOTTOM => Type::TopBottom,
            AV_STEREO3D_FRAMESEQUENCE => Type::FrameSequence,
            AV_STEREO3D_CHECKERBOARD => Type::Checkerboard,
            AV_STEREO3D_SIDEBYSIDE_QUINCUNX => Type::SideBySideQuincunx,
            AV_STEREO3D_LINES => Type::Lines,
            AV_STEREO3D_COLUMNS => Type::Columns,
            #[cfg(feature = "ffmpeg_7_1")]
            AV_STEREO3D_UNSPEC => Type::Unspecified,

            #[cfg(feature = "non-exhaustive-enums")]
            _ => unimplemented!(),
        }
    }
}

impl From<Type> for AVStereo3DType {
    #[inline(always)]
    fn from(value: Type) -> AVStereo3DType {
        match value {
            Type::TwoD => AV_STEREO3D_2D,
            Type::SideBySide => AV_STEREO3D_SIDEBYSIDE,
            Type::TopBottom => AV_STEREO3D_TOPBOTTOM,
            Type::FrameSequence => AV_STEREO3D_FRAMESEQUENCE,
            Type::Checkerboard => AV_STEREO3D_CHECKERBOARD,
            Type::SideBySideQuincunx => AV_STEREO3D_SIDEBYSIDE_QUINCUNX,
            Type::Lines => AV_STEREO3D_LINES,
            Type::Columns => AV_STEREO3D_COLUMNS,
            #[cfg(feature = "ffmpeg_7_1")]
            Type::Unspecified => AV_STEREO3D_UNSPEC,
        }
    }
}

impl Type {
    /// The safe twin of a raw `AVStereo3DType` discriminant; `None` for a
    /// value this build's libavutil does not define.
    ///
    /// `AVStereo3DType` is a `#[repr(i32)]` enum, so a value holding any other
    /// discriminant does not exist as far as the language is concerned:
    /// producing one — by casting bytes onto the struct that contains it — is
    /// undefined behaviour on the spot, before any accessor reads it. Every
    /// path from untrusted bytes to a [`Stereo3D`] goes through here and its
    /// two siblings.
    pub fn from_raw(raw: i32) -> Option<Type> {
        // Bound as `const`s so they can be used as match patterns.
        const TWO_D: i32 = AV_STEREO3D_2D as i32;
        const SIDE_BY_SIDE: i32 = AV_STEREO3D_SIDEBYSIDE as i32;
        const TOP_BOTTOM: i32 = AV_STEREO3D_TOPBOTTOM as i32;
        const FRAME_SEQUENCE: i32 = AV_STEREO3D_FRAMESEQUENCE as i32;
        const CHECKERBOARD: i32 = AV_STEREO3D_CHECKERBOARD as i32;
        const SIDE_BY_SIDE_QUINCUNX: i32 = AV_STEREO3D_SIDEBYSIDE_QUINCUNX as i32;
        const LINES: i32 = AV_STEREO3D_LINES as i32;
        const COLUMNS: i32 = AV_STEREO3D_COLUMNS as i32;
        #[cfg(feature = "ffmpeg_7_1")]
        const UNSPEC: i32 = AV_STEREO3D_UNSPEC as i32;

        match raw {
            TWO_D => Some(Type::TwoD),
            SIDE_BY_SIDE => Some(Type::SideBySide),
            TOP_BOTTOM => Some(Type::TopBottom),
            FRAME_SEQUENCE => Some(Type::FrameSequence),
            CHECKERBOARD => Some(Type::Checkerboard),
            SIDE_BY_SIDE_QUINCUNX => Some(Type::SideBySideQuincunx),
            LINES => Some(Type::Lines),
            COLUMNS => Some(Type::Columns),
            #[cfg(feature = "ffmpeg_7_1")]
            UNSPEC => Some(Type::Unspecified),
            _ => None,
        }
    }
}

/// Which views the frame actually contains — the safe twin of `AVStereo3DView`.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum View {
    /// Two packed views.
    Packed,
    /// Only the left view.
    Left,
    /// Only the right view.
    Right,
    /// Unspecified.
    #[cfg(feature = "ffmpeg_7_1")]
    Unspecified,
}

impl From<AVStereo3DView> for View {
    #[inline(always)]
    fn from(value: AVStereo3DView) -> View {
        use crate::ffi::AVStereo3DView::*;
        match value {
            AV_STEREO3D_VIEW_PACKED => View::Packed,
            AV_STEREO3D_VIEW_LEFT => View::Left,
            AV_STEREO3D_VIEW_RIGHT => View::Right,
            #[cfg(feature = "ffmpeg_7_1")]
            AV_STEREO3D_VIEW_UNSPEC => View::Unspecified,

            #[cfg(feature = "non-exhaustive-enums")]
            _ => unimplemented!(),
        }
    }
}

impl From<View> for AVStereo3DView {
    #[inline(always)]
    fn from(value: View) -> AVStereo3DView {
        use crate::ffi::AVStereo3DView::*;
        match value {
            View::Packed => AV_STEREO3D_VIEW_PACKED,
            View::Left => AV_STEREO3D_VIEW_LEFT,
            View::Right => AV_STEREO3D_VIEW_RIGHT,
            #[cfg(feature = "ffmpeg_7_1")]
            View::Unspecified => AV_STEREO3D_VIEW_UNSPEC,
        }
    }
}

impl View {
    /// The safe twin of a raw `AVStereo3DView` discriminant; `None` for a value
    /// this build's libavutil does not define. See [`Type::from_raw`].
    pub fn from_raw(raw: i32) -> Option<View> {
        use crate::ffi::AVStereo3DView::*;
        const PACKED: i32 = AV_STEREO3D_VIEW_PACKED as i32;
        const LEFT: i32 = AV_STEREO3D_VIEW_LEFT as i32;
        const RIGHT: i32 = AV_STEREO3D_VIEW_RIGHT as i32;
        #[cfg(feature = "ffmpeg_7_1")]
        const UNSPEC: i32 = AV_STEREO3D_VIEW_UNSPEC as i32;

        match raw {
            PACKED => Some(View::Packed),
            LEFT => Some(View::Left),
            RIGHT => Some(View::Right),
            #[cfg(feature = "ffmpeg_7_1")]
            UNSPEC => Some(View::Unspecified),
            _ => None,
        }
    }
}

/// Which eye to show when rendering in 2D — the safe twin of
/// `AVStereo3DPrimaryEye`.
#[cfg(feature = "ffmpeg_7_1")]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum PrimaryEye {
    /// Neither eye.
    None,
    Left,
    Right,
}

#[cfg(feature = "ffmpeg_7_1")]
impl From<AVStereo3DPrimaryEye> for PrimaryEye {
    #[inline(always)]
    fn from(value: AVStereo3DPrimaryEye) -> PrimaryEye {
        use crate::ffi::AVStereo3DPrimaryEye::*;
        match value {
            AV_PRIMARY_EYE_NONE => PrimaryEye::None,
            AV_PRIMARY_EYE_LEFT => PrimaryEye::Left,
            AV_PRIMARY_EYE_RIGHT => PrimaryEye::Right,

            #[cfg(feature = "non-exhaustive-enums")]
            _ => unimplemented!(),
        }
    }
}

#[cfg(feature = "ffmpeg_7_1")]
impl From<PrimaryEye> for AVStereo3DPrimaryEye {
    #[inline(always)]
    fn from(value: PrimaryEye) -> AVStereo3DPrimaryEye {
        use crate::ffi::AVStereo3DPrimaryEye::*;
        match value {
            PrimaryEye::None => AV_PRIMARY_EYE_NONE,
            PrimaryEye::Left => AV_PRIMARY_EYE_LEFT,
            PrimaryEye::Right => AV_PRIMARY_EYE_RIGHT,
        }
    }
}

#[cfg(feature = "ffmpeg_7_1")]
impl PrimaryEye {
    /// The safe twin of a raw `AVStereo3DPrimaryEye` discriminant; `None` for a
    /// value this build's libavutil does not define. See [`Type::from_raw`].
    pub fn from_raw(raw: i32) -> Option<PrimaryEye> {
        use crate::ffi::AVStereo3DPrimaryEye::*;
        const NONE: i32 = AV_PRIMARY_EYE_NONE as i32;
        const LEFT: i32 = AV_PRIMARY_EYE_LEFT as i32;
        const RIGHT: i32 = AV_PRIMARY_EYE_RIGHT as i32;

        match raw {
            NONE => Some(PrimaryEye::None),
            LEFT => Some(PrimaryEye::Left),
            RIGHT => Some(PrimaryEye::Right),
            _ => None,
        }
    }
}

/// A borrowed `AVStereo3D` — the payload of `AV_PKT_DATA_STEREO3D` /
/// `AV_FRAME_DATA_STEREO3D` side data.
///
/// Every field is exposed: beyond the packing type and the flags, a stereo
/// renderer needs the view, the primary eye, the baseline and the two
/// adjustment rationals to decide what to show.
pub struct Stereo3D<'a> {
    ptr: *const AVStereo3D,

    _marker: PhantomData<&'a AVStereo3D>,
}

impl<'a> Stereo3D<'a> {
    /// # Safety
    ///
    /// `ptr` must point at a live, correctly aligned `AVStereo3D` that outlives
    /// `'a` and whose enum-typed fields — `type_`, `view` and (on FFmpeg 7.1+)
    /// `primary_eye` — each hold a discriminant their enum defines; see
    /// [`Type::from_raw`]. A pointer libavutil produced satisfies all of it.
    pub unsafe fn wrap(ptr: *const AVStereo3D) -> Self {
        Stereo3D {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Borrow a side-data payload as an `AVStereo3D`.
    ///
    /// `None` when the buffer cannot be one:
    ///
    /// * it is shorter than the struct;
    /// * it is not aligned for the struct;
    /// * one of the enum-typed fields (`type_`, `view`, `primary_eye`) holds a
    ///   value its enum does not define, which is screened from the raw `i32`s
    ///   here because the cast itself would already be undefined behaviour.
    ///
    /// libavutil allocates the payload with `av_stereo3d_alloc_size` and writes
    /// only defined values into it, so a genuine side-data buffer always
    /// passes.
    pub fn from_side_data(data: &'a [u8]) -> Option<Self> {
        if data.len() < mem::size_of::<AVStereo3D>() {
            return None;
        }
        let ptr = data.as_ptr() as *const AVStereo3D;
        if !ptr.is_aligned() {
            return None;
        }
        // SAFETY (every read below): `data` is long enough for the whole
        // struct and aligned for it, so each of these places is a readable,
        // aligned `i32`. `&raw const` takes the address without loading it at
        // the declared enum type — that load is exactly the undefined
        // behaviour being screened for.
        let type_ = unsafe { (&raw const (*ptr).type_).cast::<i32>().read() };
        Type::from_raw(type_)?;
        let view = unsafe { (&raw const (*ptr).view).cast::<i32>().read() };
        View::from_raw(view)?;
        #[cfg(feature = "ffmpeg_7_1")]
        {
            let primary_eye = unsafe { (&raw const (*ptr).primary_eye).cast::<i32>().read() };
            PrimaryEye::from_raw(primary_eye)?;
        }
        // SAFETY: size, alignment and every enum-typed field are checked above,
        // so `wrap`'s contract holds, and the borrow of `data` is carried into
        // the returned lifetime.
        Some(unsafe { Self::wrap(ptr) })
    }

    pub fn as_ptr(&self) -> *const AVStereo3D {
        self.ptr
    }

    /// How the views are packed.
    pub fn kind(&self) -> Type {
        unsafe { Type::from((*self.as_ptr()).type_) }
    }

    /// `AV_STEREO3D_FLAG_INVERT`: the right/bottom half holds the LEFT view.
    pub fn inverted(&self) -> bool {
        unsafe { (*self.as_ptr()).flags & AV_STEREO3D_FLAG_INVERT != 0 }
    }

    /// Which views the frame contains.
    pub fn view(&self) -> View {
        unsafe { View::from((*self.as_ptr()).view) }
    }

    /// Which eye is primary when rendering in 2D.
    #[cfg(feature = "ffmpeg_7_1")]
    pub fn primary_eye(&self) -> PrimaryEye {
        unsafe { PrimaryEye::from((*self.as_ptr()).primary_eye) }
    }

    /// Distance between the lens centres, micrometres. `0` = unset.
    #[cfg(feature = "ffmpeg_7_1")]
    pub fn baseline(&self) -> u32 {
        unsafe { (*self.as_ptr()).baseline }
    }

    /// Relative shift of the two images, -1.0..1.0. Zero = unset.
    #[cfg(feature = "ffmpeg_7_1")]
    pub fn horizontal_disparity_adjustment(&self) -> Rational {
        unsafe { Rational::from((*self.as_ptr()).horizontal_disparity_adjustment) }
    }

    /// Horizontal field of view, degrees. Zero = unset.
    #[cfg(feature = "ffmpeg_7_1")]
    pub fn horizontal_field_of_view(&self) -> Rational {
        unsafe { Rational::from((*self.as_ptr()).horizontal_field_of_view) }
    }
}
