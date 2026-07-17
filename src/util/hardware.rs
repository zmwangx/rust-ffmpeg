use std::convert::TryFrom;
use std::ffi::CString;
use std::ptr;

use ffi::*;
use libc::c_int;
use {format, frame, Error};

/// A reference-counted FFmpeg hardware device context.
pub struct Device {
    ptr: *mut AVBufferRef,
}

impl Device {
    /// Creates a hardware device context.
    ///
    /// `device` is backend-specific. For example, VAAPI commonly uses
    /// `/dev/dri/renderD128`; pass `None` to let FFmpeg choose its default.
    #[inline]
    pub fn create(kind: AVHWDeviceType, device: Option<&str>) -> Result<Self, Error> {
        let device = device
            .map(CString::new)
            .transpose()
            .map_err(|_| Error::InvalidData)?;
        let mut ptr = ptr::null_mut();
        let result = unsafe {
            av_hwdevice_ctx_create(
                &mut ptr,
                kind,
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
        Frames::new(self, format, software_format, width, height)
    }

    #[inline(always)]
    pub(crate) fn as_ptr(&self) -> *mut AVBufferRef {
        self.ptr
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
    fn new(
        device: &Device,
        format: format::Pixel,
        software_format: format::Pixel,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let width = c_int::try_from(width).map_err(|_| Error::InvalidData)?;
        let height = c_int::try_from(height).map_err(|_| Error::InvalidData)?;
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
        destination.set_pts(source.pts());
        let result =
            unsafe { av_hwframe_transfer_data(destination.as_mut_ptr(), source.as_ptr(), 0) };
        match result {
            e if e < 0 => Err(Error::from(e)),
            _ => Ok(destination),
        }
    }

    #[inline]
    pub(crate) fn try_clone_raw(&self) -> Result<*mut AVBufferRef, Error> {
        let ptr = unsafe { av_buffer_ref(self.ptr) };
        if ptr.is_null() {
            Err(Error::Other {
                errno: libc::ENOMEM,
            })
        } else {
            Ok(ptr)
        }
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        unsafe { av_buffer_unref(&mut self.ptr) };
    }
}
