use std::convert::TryFrom;
use std::ffi::CString;
use std::ptr;

use ffi::*;
use libc::c_int;
use {format, frame, Error};

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

fn clone_buffer(ptr: *mut AVBufferRef) -> Result<*mut AVBufferRef, Error> {
    if ptr.is_null() {
        return Err(Error::InvalidData);
    }

    let ptr = unsafe { av_buffer_ref(ptr) };
    if ptr.is_null() {
        Err(Error::Other {
            errno: libc::ENOMEM,
        })
    } else {
        Ok(ptr)
    }
}

/// A reference-counted FFmpeg hardware device context.
pub struct Device {
    ptr: *mut AVBufferRef,
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
        if ptr.is_null() {
            Err(Error::InvalidData)
        } else {
            Ok(Device { ptr })
        }
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

    /// Creates a hardware device context.
    ///
    /// `device` is backend-specific. For example, VAAPI commonly uses
    /// `/dev/dri/renderD128`; pass `None` to let FFmpeg choose its default.
    #[inline]
    pub fn create(kind: Type, device: Option<&str>) -> Result<Self, Error> {
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
                ptr::null_mut(),
                0,
            )
        };

        match result {
            e if e < 0 => Err(Error::from(e)),
            _ => Ok(Device { ptr }),
        }
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

    #[inline(always)]
    pub(crate) fn as_ptr(&self) -> *mut AVBufferRef {
        self.ptr
    }

    #[inline]
    pub fn kind(&self) -> Result<Type, Error> {
        let context = unsafe { (*self.ptr).data.cast::<AVHWDeviceContext>() };
        if context.is_null() {
            Err(Error::InvalidData)
        } else {
            Type::try_from(unsafe { (*context).type_ })
        }
    }

    #[inline]
    pub(crate) fn try_clone_raw(&self) -> Result<*mut AVBufferRef, Error> {
        clone_buffer(self.ptr)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { av_buffer_unref(&mut self.ptr) };
    }
}

/// A reference-counted pool of hardware frames tied to a [`Device`].
pub struct Frames {
    ptr: *mut AVBufferRef,
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
        if ptr.is_null() {
            Err(Error::InvalidData)
        } else {
            Ok(Frames { ptr })
        }
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
        let mut ptr = unsafe { av_hwframe_ctx_alloc(device.as_ptr()) };
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
            _ => Ok(Frames { ptr }),
        }
    }

    /// Allocates a video frame from this hardware frame pool.
    #[inline]
    pub fn allocate_video(&self) -> Result<frame::Video, Error> {
        let mut frame = frame::Video::empty();
        let result = unsafe { av_hwframe_get_buffer(self.ptr, frame.as_mut_ptr(), 0) };
        match result {
            e if e < 0 => Err(Error::from(e)),
            _ => Ok(frame),
        }
    }

    /// Uploads a software frame into a frame allocated from this pool.
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

    /// Downloads a hardware frame into software memory.
    #[inline]
    pub fn download(&self, source: &frame::Video) -> Result<frame::Video, Error> {
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

    #[inline]
    fn context(&self) -> &AVHWFramesContext {
        unsafe { &*(*self.ptr).data.cast::<AVHWFramesContext>() }
    }

    #[inline]
    pub fn format(&self) -> format::Pixel {
        self.context().format.into()
    }

    #[inline]
    pub fn software_format(&self) -> format::Pixel {
        self.context().sw_format.into()
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.context().width as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.context().height as u32
    }

    #[inline]
    pub fn initial_pool_size(&self) -> usize {
        self.context().initial_pool_size as usize
    }

    /// Gets the device backing this hardware frame pool.
    #[inline]
    pub fn device(&self) -> Result<Device, Error> {
        clone_buffer(self.context().device_ref).map(|ptr| Device { ptr })
    }

    #[inline]
    pub(crate) fn try_clone_raw(&self) -> Result<*mut AVBufferRef, Error> {
        clone_buffer(self.ptr)
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        unsafe { av_buffer_unref(&mut self.ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::{Frames, Type};
    use ffi::AVHWDeviceType;
    use frame;
    use std::convert::TryFrom;

    #[test]
    fn software_frame_has_no_hardware_frames_context() {
        let frame = frame::Video::empty();

        assert!(Frames::from_video(&frame).unwrap().is_none());
    }

    #[test]
    fn safe_device_type_roundtrips_through_ffi() {
        let kind = Type::Vaapi;
        let raw: AVHWDeviceType = kind.into();

        assert_eq!(Type::try_from(raw).unwrap(), kind);
    }
}
