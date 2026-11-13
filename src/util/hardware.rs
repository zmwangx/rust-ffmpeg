use std::convert::TryFrom;
use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::ptr::{self, NonNull};

use crate::ffi::*;
use crate::{DictionaryRef, Error, format, frame};
use libc::c_int;

/// A hardware device backend supported by FFmpeg.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Vdpau,
    Cuda,
    Vaapi,
    Dxva2,
    Qsv,
    VideoToolbox,
    D3d11va,
    Drm,
    #[cfg(feature = "ffmpeg_4_0")]
    OpenCl,
    #[cfg(feature = "ffmpeg_4_0")]
    MediaCodec,
    #[cfg(feature = "ffmpeg_4_3")]
    Vulkan,
    #[cfg(feature = "ffmpeg_7_0")]
    D3d12va,
    #[cfg(feature = "ffmpeg_8_1")]
    Amf,
    #[cfg(feature = "ffmpeg_8_0")]
    OhCodec,
}

impl From<Type> for AVHWDeviceType {
    fn from(value: Type) -> Self {
        match value {
            Type::Vdpau => AVHWDeviceType::AV_HWDEVICE_TYPE_VDPAU,
            Type::Cuda => AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA,
            Type::Vaapi => AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI,
            Type::Dxva2 => AVHWDeviceType::AV_HWDEVICE_TYPE_DXVA2,
            Type::Qsv => AVHWDeviceType::AV_HWDEVICE_TYPE_QSV,
            Type::VideoToolbox => AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX,
            Type::D3d11va => AVHWDeviceType::AV_HWDEVICE_TYPE_D3D11VA,
            Type::Drm => AVHWDeviceType::AV_HWDEVICE_TYPE_DRM,
            #[cfg(feature = "ffmpeg_4_0")]
            Type::OpenCl => AVHWDeviceType::AV_HWDEVICE_TYPE_OPENCL,
            #[cfg(feature = "ffmpeg_4_0")]
            Type::MediaCodec => AVHWDeviceType::AV_HWDEVICE_TYPE_MEDIACODEC,
            #[cfg(feature = "ffmpeg_4_3")]
            Type::Vulkan => AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN,
            #[cfg(feature = "ffmpeg_7_0")]
            Type::D3d12va => AVHWDeviceType::AV_HWDEVICE_TYPE_D3D12VA,
            #[cfg(feature = "ffmpeg_8_1")]
            Type::Amf => AVHWDeviceType::AV_HWDEVICE_TYPE_AMF,
            #[cfg(feature = "ffmpeg_8_0")]
            Type::OhCodec => AVHWDeviceType::AV_HWDEVICE_TYPE_OHCODEC,
        }
    }
}

impl TryFrom<AVHWDeviceType> for Type {
    type Error = Error;

    fn try_from(value: AVHWDeviceType) -> Result<Self, Self::Error> {
        match value {
            AVHWDeviceType::AV_HWDEVICE_TYPE_VDPAU => Ok(Type::Vdpau),
            AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA => Ok(Type::Cuda),
            AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI => Ok(Type::Vaapi),
            AVHWDeviceType::AV_HWDEVICE_TYPE_DXVA2 => Ok(Type::Dxva2),
            AVHWDeviceType::AV_HWDEVICE_TYPE_QSV => Ok(Type::Qsv),
            AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX => Ok(Type::VideoToolbox),
            AVHWDeviceType::AV_HWDEVICE_TYPE_D3D11VA => Ok(Type::D3d11va),
            AVHWDeviceType::AV_HWDEVICE_TYPE_DRM => Ok(Type::Drm),
            #[cfg(feature = "ffmpeg_4_0")]
            AVHWDeviceType::AV_HWDEVICE_TYPE_OPENCL => Ok(Type::OpenCl),
            #[cfg(feature = "ffmpeg_4_0")]
            AVHWDeviceType::AV_HWDEVICE_TYPE_MEDIACODEC => Ok(Type::MediaCodec),
            #[cfg(feature = "ffmpeg_4_3")]
            AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN => Ok(Type::Vulkan),
            #[cfg(feature = "ffmpeg_7_0")]
            AVHWDeviceType::AV_HWDEVICE_TYPE_D3D12VA => Ok(Type::D3d12va),
            #[cfg(feature = "ffmpeg_8_1")]
            AVHWDeviceType::AV_HWDEVICE_TYPE_AMF => Ok(Type::Amf),
            #[cfg(feature = "ffmpeg_8_0")]
            AVHWDeviceType::AV_HWDEVICE_TYPE_OHCODEC => Ok(Type::OhCodec),
            _ => Err(Error::InvalidData),
        }
    }
}

fn clone_buffer(ptr: *mut AVBufferRef) -> Result<NonNull<AVBufferRef>, Error> {
    if ptr.is_null() {
        return Err(Error::InvalidData);
    }

    let ptr = unsafe { av_buffer_ref(ptr) };
    NonNull::new(ptr).ok_or(Error::Other {
        errno: libc::ENOMEM,
    })
}

fn created_buffer(result: c_int, ptr: *mut AVBufferRef) -> Result<NonNull<AVBufferRef>, Error> {
    if result < 0 {
        Err(Error::from(result))
    } else {
        NonNull::new(ptr).ok_or(Error::InvalidData)
    }
}

/// A reference-counted FFmpeg hardware device context.
pub struct Device {
    ptr: NonNull<AVBufferRef>,
}

impl Device {
    /// Takes ownership of an existing hardware device buffer reference.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid, owned reference to an initialized
    /// `AVHWDeviceContext`. This method consumes that reference.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut AVBufferRef) -> Result<Self, Error> {
        NonNull::new(ptr)
            .map(|ptr| Device { ptr })
            .ok_or(Error::InvalidData)
    }

    /// Wraps an existing hardware device context by taking a new reference.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, initialized `AVHWDeviceContext` buffer
    /// reference for the duration of this call.
    #[inline]
    pub unsafe fn wrap(ptr: *mut AVBufferRef) -> Result<Self, Error> {
        clone_buffer(ptr).map(|ptr| Device { ptr })
    }

    /// Returns the underlying FFmpeg buffer reference without transferring
    /// ownership.
    ///
    /// The pointer is valid only while this wrapper remains alive. Callers
    /// must not unreference it unless they first create their own reference.
    #[inline(always)]
    pub unsafe fn as_ptr(&self) -> *mut AVBufferRef {
        self.ptr.as_ptr()
    }

    /// Transfers ownership of the underlying FFmpeg buffer reference.
    ///
    /// The caller becomes responsible for eventually releasing the returned
    /// reference with `av_buffer_unref()` or transferring it to FFmpeg.
    #[inline]
    pub fn into_raw(self) -> *mut AVBufferRef {
        ManuallyDrop::new(self).ptr.as_ptr()
    }

    /// Creates another owned reference to this device context.
    #[inline]
    pub fn try_clone(&self) -> Result<Self, Error> {
        clone_buffer(self.ptr.as_ptr()).map(|ptr| Device { ptr })
    }

    /// Creates a hardware device context.
    ///
    /// `device` is backend-specific. For example, VAAPI commonly uses
    /// `/dev/dri/renderD128`; pass `None` to let FFmpeg choose its default.
    #[inline]
    pub fn create(kind: Type, device: Option<&str>) -> Result<Self, Error> {
        Self::create_inner(kind, device, ptr::null_mut())
    }

    /// Creates a hardware device context with backend-specific options.
    #[inline]
    pub fn create_with_options(
        kind: Type,
        device: Option<&str>,
        options: &DictionaryRef<'_>,
    ) -> Result<Self, Error> {
        let options = unsafe { options.as_ptr() as *mut AVDictionary };
        Self::create_inner(kind, device, options)
    }

    fn create_inner(
        kind: Type,
        device: Option<&str>,
        options: *mut AVDictionary,
    ) -> Result<Self, Error> {
        let device = device
            .map(CString::new)
            .transpose()
            .map_err(|_| Error::InvalidData)?;
        let mut ptr = ptr::null_mut();
        let result = unsafe {
            av_hwdevice_ctx_create(
                &mut ptr,
                kind.into(),
                device.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                options,
                0,
            )
        };

        created_buffer(result, ptr).map(|ptr| Device { ptr })
    }

    /// Creates or retrieves a device of `kind` derived from this device.
    #[inline]
    pub fn derive(&self, kind: Type) -> Result<Self, Error> {
        let mut ptr = ptr::null_mut();
        let result =
            unsafe { av_hwdevice_ctx_create_derived(&mut ptr, kind.into(), self.ptr.as_ptr(), 0) };

        created_buffer(result, ptr).map(|ptr| Device { ptr })
    }

    /// Creates or retrieves a derived device with backend-specific options.
    #[cfg(feature = "ffmpeg_4_4")]
    #[inline]
    pub fn derive_with_options(
        &self,
        kind: Type,
        options: &DictionaryRef<'_>,
    ) -> Result<Self, Error> {
        let mut ptr = ptr::null_mut();
        let options = unsafe { options.as_ptr() as *mut AVDictionary };
        let result = unsafe {
            av_hwdevice_ctx_create_derived_opts(
                &mut ptr,
                kind.into(),
                self.ptr.as_ptr(),
                options,
                0,
            )
        };

        created_buffer(result, ptr).map(|ptr| Device { ptr })
    }

    /// Creates a hardware frame pool backed by this device.
    #[inline]
    pub fn frames(
        &self,
        format: format::Pixel,
        software_format: format::Pixel,
        width: u32,
        height: u32,
    ) -> Result<Frames, Error> {
        Frames::new(self, format, software_format, width, height, 0)
    }

    /// Creates a hardware frame pool with a requested initial pool size.
    ///
    /// Some hardware backends use fixed-size pools and require this value to
    /// be large enough for the encoder or decoder's in-flight frames.
    #[inline]
    pub fn frames_with_pool_size(
        &self,
        format: format::Pixel,
        software_format: format::Pixel,
        width: u32,
        height: u32,
        initial_pool_size: usize,
    ) -> Result<Frames, Error> {
        Frames::new(
            self,
            format,
            software_format,
            width,
            height,
            initial_pool_size,
        )
    }

    #[inline]
    pub fn kind(&self) -> Result<Type, Error> {
        let context = unsafe { (*self.ptr.as_ptr()).data.cast::<AVHWDeviceContext>() };
        if context.is_null() {
            Err(Error::InvalidData)
        } else {
            Type::try_from(unsafe { (*context).type_ })
        }
    }

    #[inline]
    pub(crate) fn try_clone_raw(&self) -> Result<*mut AVBufferRef, Error> {
        self.try_clone().map(Device::into_raw)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let mut ptr = self.ptr.as_ptr();
        unsafe { av_buffer_unref(&mut ptr) };
    }
}

/// A reference-counted pool of hardware frames tied to a [`Device`].
pub struct Frames {
    ptr: NonNull<AVBufferRef>,
}

impl Frames {
    /// Takes ownership of an existing hardware frames buffer reference.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid, owned reference to an initialized
    /// `AVHWFramesContext`. This method consumes that reference.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut AVBufferRef) -> Result<Self, Error> {
        NonNull::new(ptr)
            .map(|ptr| Frames { ptr })
            .ok_or(Error::InvalidData)
    }

    /// Wraps an existing hardware frames context by taking a new reference.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, initialized `AVHWFramesContext` buffer
    /// reference for the duration of this call.
    #[inline]
    pub unsafe fn wrap(ptr: *mut AVBufferRef) -> Result<Self, Error> {
        clone_buffer(ptr).map(|ptr| Frames { ptr })
    }

    /// Returns the underlying FFmpeg buffer reference without transferring
    /// ownership.
    ///
    /// The pointer is valid only while this wrapper remains alive. Callers
    /// must not unreference it unless they first create their own reference.
    #[inline(always)]
    pub unsafe fn as_ptr(&self) -> *mut AVBufferRef {
        self.ptr.as_ptr()
    }

    /// Transfers ownership of the underlying FFmpeg buffer reference.
    ///
    /// The caller becomes responsible for eventually releasing the returned
    /// reference with `av_buffer_unref()` or transferring it to FFmpeg.
    #[inline]
    pub fn into_raw(self) -> *mut AVBufferRef {
        ManuallyDrop::new(self).ptr.as_ptr()
    }

    /// Creates another owned reference to this hardware frame pool.
    #[inline]
    pub fn try_clone(&self) -> Result<Self, Error> {
        clone_buffer(self.ptr.as_ptr()).map(|ptr| Frames { ptr })
    }

    /// Gets the hardware frames context attached to a decoded frame.
    ///
    /// The returned wrapper owns a new reference, so it remains valid after
    /// `source` is reused or dropped. It can be passed directly to a hardware
    /// encoder for a zero-copy decode-to-encode path.
    #[inline]
    pub fn from_video(source: &frame::Video) -> Result<Option<Self>, Error> {
        let ptr = unsafe { (*source.as_ptr()).hw_frames_ctx };
        if ptr.is_null() {
            Ok(None)
        } else {
            clone_buffer(ptr).map(|ptr| Some(Frames { ptr }))
        }
    }

    fn new(
        device: &Device,
        format: format::Pixel,
        software_format: format::Pixel,
        width: u32,
        height: u32,
        initial_pool_size: usize,
    ) -> Result<Self, Error> {
        let width = c_int::try_from(width).map_err(|_| Error::InvalidData)?;
        let height = c_int::try_from(height).map_err(|_| Error::InvalidData)?;
        let initial_pool_size =
            c_int::try_from(initial_pool_size).map_err(|_| Error::InvalidData)?;
        let mut ptr = unsafe { av_hwframe_ctx_alloc(device.ptr.as_ptr()) };
        if ptr.is_null() {
            return Err(Error::Other {
                errno: libc::ENOMEM,
            });
        }

        let context = unsafe { (*ptr).data.cast::<AVHWFramesContext>() };
        if context.is_null() {
            unsafe { av_buffer_unref(&mut ptr) };
            return Err(Error::InvalidData);
        }
        unsafe {
            (*context).format = format.into();
            (*context).sw_format = software_format.into();
            (*context).width = width;
            (*context).height = height;
            (*context).initial_pool_size = initial_pool_size;
        }

        let result = unsafe { av_hwframe_ctx_init(ptr) };
        match result {
            e if e < 0 => {
                unsafe { av_buffer_unref(&mut ptr) };
                Err(Error::from(e))
            }
            _ => NonNull::new(ptr)
                .map(|ptr| Frames { ptr })
                .ok_or(Error::InvalidData),
        }
    }

    /// Allocates a video frame from this hardware frame pool.
    #[inline]
    pub fn allocate_video(&self) -> Result<frame::Video, Error> {
        let mut frame = frame::Video::empty();
        let result = unsafe { av_hwframe_get_buffer(self.ptr.as_ptr(), frame.as_mut_ptr(), 0) };
        match result {
            e if e < 0 => Err(Error::from(e)),
            _ => Ok(frame),
        }
    }

    /// Uploads a software frame into a frame allocated from this pool.
    ///
    /// The source format must be compatible with [`Frames::software_format`],
    /// and its allocated dimensions must match this pool. FFmpeg validates the
    /// backend-specific transfer constraints.
    #[inline]
    pub fn upload(&self, source: &frame::Video) -> Result<frame::Video, Error> {
        let mut destination = self.allocate_video()?;
        let result =
            unsafe { av_hwframe_transfer_data(destination.as_mut_ptr(), source.as_ptr(), 0) };
        if result < 0 {
            return Err(Error::from(result));
        }

        let result = unsafe { av_frame_copy_props(destination.as_mut_ptr(), source.as_ptr()) };
        if result < 0 {
            Err(Error::from(result))
        } else {
            Ok(destination)
        }
    }

    /// Downloads a hardware frame from this pool into software memory.
    ///
    /// `source` must carry an `AVHWFramesContext` referring to this pool. The
    /// destination uses this pool's software format and the source frame's
    /// display dimensions.
    #[inline]
    pub fn download(&self, source: &frame::Video) -> Result<frame::Video, Error> {
        if !self.contains(source) {
            return Err(Error::InvalidData);
        }

        let mut destination = frame::Video::empty();
        destination.set_format(self.software_format());
        destination.set_width(source.width());
        destination.set_height(source.height());

        let result =
            unsafe { av_hwframe_transfer_data(destination.as_mut_ptr(), source.as_ptr(), 0) };
        if result < 0 {
            return Err(Error::from(result));
        }

        let result = unsafe { av_frame_copy_props(destination.as_mut_ptr(), source.as_ptr()) };
        if result < 0 {
            Err(Error::from(result))
        } else {
            Ok(destination)
        }
    }

    /// Returns whether `frame` was allocated from this hardware frame pool.
    #[inline]
    pub fn contains(&self, frame: &frame::Video) -> bool {
        let frame_context = unsafe { (*frame.as_ptr()).hw_frames_ctx };
        if frame_context.is_null() {
            return false;
        }

        unsafe { (*frame_context).data == (*self.ptr.as_ptr()).data }
    }

    #[inline]
    fn context_ptr(&self) -> *const AVHWFramesContext {
        unsafe { (*self.ptr.as_ptr()).data.cast::<AVHWFramesContext>() }
    }

    #[inline]
    pub fn format(&self) -> format::Pixel {
        unsafe { (*self.context_ptr()).format.into() }
    }

    #[inline]
    pub fn software_format(&self) -> format::Pixel {
        unsafe { (*self.context_ptr()).sw_format.into() }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        unsafe { (*self.context_ptr()).width as u32 }
    }

    #[inline]
    pub fn height(&self) -> u32 {
        unsafe { (*self.context_ptr()).height as u32 }
    }

    #[inline]
    pub fn initial_pool_size(&self) -> usize {
        unsafe { (*self.context_ptr()).initial_pool_size as usize }
    }

    /// Gets the device backing this hardware frame pool.
    #[inline]
    pub fn device(&self) -> Result<Device, Error> {
        let device = unsafe { (*self.context_ptr()).device_ref };
        clone_buffer(device).map(|ptr| Device { ptr })
    }

    #[inline]
    pub(crate) fn try_clone_raw(&self) -> Result<*mut AVBufferRef, Error> {
        self.try_clone().map(Frames::into_raw)
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        let mut ptr = self.ptr.as_ptr();
        unsafe { av_buffer_unref(&mut ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::{Device, Frames, Type};
    use crate::ffi::{AVBufferRef, AVHWDeviceType};
    use crate::frame;
    use std::convert::TryFrom;
    use std::ptr;

    #[test]
    fn software_frame_has_no_hardware_frames_context() {
        let frame = frame::Video::empty();

        assert!(Frames::from_video(&frame).unwrap().is_none());
    }

    #[test]
    fn safe_device_type_roundtrips_through_ffi() {
        let kinds = [
            Type::Vdpau,
            Type::Cuda,
            Type::Vaapi,
            Type::Dxva2,
            Type::Qsv,
            Type::VideoToolbox,
            Type::D3d11va,
            Type::Drm,
            #[cfg(feature = "ffmpeg_4_0")]
            Type::OpenCl,
            #[cfg(feature = "ffmpeg_4_0")]
            Type::MediaCodec,
            #[cfg(feature = "ffmpeg_4_3")]
            Type::Vulkan,
            #[cfg(feature = "ffmpeg_7_0")]
            Type::D3d12va,
            #[cfg(feature = "ffmpeg_8_1")]
            Type::Amf,
            #[cfg(feature = "ffmpeg_8_0")]
            Type::OhCodec,
        ];

        for kind in kinds {
            let raw: AVHWDeviceType = kind.into();

            assert_eq!(Type::try_from(raw).unwrap(), kind);
        }
    }

    #[test]
    fn raw_constructors_reject_null_references() {
        let ptr = ptr::null_mut::<AVBufferRef>();

        assert!(unsafe { Device::from_raw(ptr) }.is_err());
        assert!(unsafe { Device::wrap(ptr) }.is_err());
        assert!(unsafe { Frames::from_raw(ptr) }.is_err());
        assert!(unsafe { Frames::wrap(ptr) }.is_err());
    }
}
