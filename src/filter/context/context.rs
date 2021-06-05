use std::marker::PhantomData;

use super::{Sink, Source};
use ffi::*;
use libc::c_void;
use {format, option, ChannelLayout, Error};

pub struct Context<'a> {
    ptr: *mut AVFilterContext,

    _marker: PhantomData<&'a ()>,
}

impl<'a> Context<'a> {
    pub unsafe fn wrap(ptr: *mut AVFilterContext) -> Self {
        Context {
            ptr,
            _marker: PhantomData,
        }
    }

    pub unsafe fn as_ptr(&self) -> *const AVFilterContext {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFilterContext {
        self.ptr
    }
}

impl<'a> Context<'a> {
    pub fn source(&'a mut self) -> Source<'a> {
        unsafe { Source::wrap(self) }
    }

    pub fn sink(&'a mut self) -> Sink<'a> {
        unsafe { Sink::wrap(self) }
    }

    pub fn set_pixel_format(&mut self, value: format::Pixel) {
        let _ = option::Settable::set::<AVPixelFormat>(self, "pix_fmts", &value.into());
    }

    pub fn set_sample_format(&mut self, value: format::Sample) {
        let _ = option::Settable::set::<AVSampleFormat>(self, "sample_fmts", &value.into());
    }

    pub fn set_sample_rate(&mut self, value: u32) {
        let _ = option::Settable::set(self, "sample_rates", &i64::from(value));
    }

    pub fn set_channel_layout(&mut self, value: ChannelLayout) {
        let _ = option::Settable::set(self, "channel_layouts", &value.bits());
    }

    /// Links this filter context to another one.
    ///
    /// For more info view [ffmpeg's documentation](
    /// https://ffmpeg.org/doxygen/3.4/group__lavfi.html#gabc6247ebae2c591e768c8555174402f1).
    pub fn link<'b, 'c>(&mut self, other: &'b mut Context<'c>) -> Result<&'b mut Context<'c>, Error>
    where
        'a: 'b,
        'b: 'c,
    {
        unsafe {
            match sys::avfilter_link(self.as_mut_ptr(), 0, other.as_mut_ptr(), 0) {
                s if s >= 0 => Ok(other),
                e => Err(Error::from(e)),
            }
        }
    }
}

unsafe impl<'a> option::Target for Context<'a> {
    fn as_ptr(&self) -> *const c_void {
        self.ptr as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        self.ptr as *mut _
    }
}

impl<'a> option::Settable for Context<'a> {}
