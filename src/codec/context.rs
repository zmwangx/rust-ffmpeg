use std::any::Any;
use std::ptr;
use std::rc::Rc;

use super::decoder::Decoder;
use super::encoder::Encoder;
use super::{threading, Compliance, Debug, Flags, Id, Parameters};
use ffi::*;
use libc::c_int;
use media;
use {Codec, Error};

pub struct Context {
    ptr: *mut AVCodecContext,
    owner: Option<Rc<dyn Any>>,
}

unsafe impl Send for Context {}

impl Context {
    pub unsafe fn wrap(ptr: *mut AVCodecContext, owner: Option<Rc<dyn Any>>) -> Self {
        Context { ptr, owner }
    }

    pub unsafe fn as_ptr(&self) -> *const AVCodecContext {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.ptr
    }
}

impl Context {
    pub fn new() -> Self {
        unsafe {
            Context {
                ptr: avcodec_alloc_context3(ptr::null()),
                owner: None,
            }
        }
    }

    pub fn from_parameters<P: Into<Parameters>>(parameters: P) -> Result<Self, Error> {
        let parameters = parameters.into();
        let mut context = Self::new();

        unsafe {
            match avcodec_parameters_to_context(context.as_mut_ptr(), parameters.as_ptr()) {
                e if e < 0 => Err(Error::from(e)),
                _ => Ok(context),
            }
        }
    }

    pub fn decoder(self) -> Decoder {
        Decoder(self)
    }

    pub fn encoder(self) -> Encoder {
        Encoder(self)
    }

    pub fn codec(&self) -> Option<Codec> {
        unsafe {
            if (*self.as_ptr()).codec.is_null() {
                None
            } else {
                Some(Codec::wrap((*self.as_ptr()).codec as *mut _))
            }
        }
    }

    pub fn medium(&self) -> media::Type {
        unsafe { media::Type::from((*self.as_ptr()).codec_type) }
    }

    pub fn set_flags(&mut self, value: Flags) {
        unsafe {
            (*self.as_mut_ptr()).flags = value.bits() as c_int;
        }
    }

    pub fn id(&self) -> Id {
        unsafe { Id::from((*self.as_ptr()).codec_id) }
    }

    pub fn compliance(&mut self, value: Compliance) {
        unsafe {
            (*self.as_mut_ptr()).strict_std_compliance = value.into();
        }
    }

    pub fn debug(&mut self, value: Debug) {
        unsafe {
            (*self.as_mut_ptr()).debug = value.bits();
        }
    }

    pub fn set_threading(&mut self, config: threading::Config) {
        unsafe {
            (*self.as_mut_ptr()).thread_type = config.kind.into();
            (*self.as_mut_ptr()).thread_count = config.count as c_int;
            (*self.as_mut_ptr()).thread_safe_callbacks = if config.safe { 1 } else { 0 };
        }
    }

    pub fn threading(&self) -> threading::Config {
        unsafe {
            threading::Config {
                kind: threading::Type::from((*self.as_ptr()).active_thread_type),
                count: (*self.as_ptr()).thread_count as usize,
                safe: (*self.as_ptr()).thread_safe_callbacks != 0,
            }
        }
    }

    pub fn set_parameters<P: Into<Parameters>>(&mut self, parameters: P) -> Result<(), Error> {
        let parameters = parameters.into();

        unsafe {
            match avcodec_parameters_to_context(self.as_mut_ptr(), parameters.as_ptr()) {
                e if e < 0 => Err(Error::from(e)),
                _ => Ok(()),
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            if self.owner.is_none() {
                avcodec_free_context(&mut self.as_mut_ptr());
            }
        }
    }
}

#[cfg(not(feature = "ffmpeg_5_0"))]
impl Clone for Context {
    fn clone(&self) -> Self {
        let mut ctx = Context::new();
        ctx.clone_from(self);

        ctx
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            // Removed in ffmpeg >= 5.0.
            avcodec_copy_context(self.as_mut_ptr(), source.as_ptr());
        }
    }
}
