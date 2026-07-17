use std::any::Any;
use std::sync::Arc;

use super::{Context, Id};
use ffi::*;
use media;

pub struct Parameters {
    ptr: *mut AVCodecParameters,
    owner: Option<Arc<dyn Any + Send + Sync>>,
}

// SAFETY: the parameters either own their `AVCodecParameters` or keep their
// owner alive through an atomically refcounted, thread-safe handle.
unsafe impl Send for Parameters {}

impl Parameters {
    pub unsafe fn wrap(
        ptr: *mut AVCodecParameters,
        owner: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Self {
        Parameters { ptr, owner }
    }

    pub unsafe fn as_ptr(&self) -> *const AVCodecParameters {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.ptr
    }
}

impl Parameters {
    pub fn new() -> Self {
        unsafe {
            Parameters {
                ptr: avcodec_parameters_alloc(),
                owner: None,
            }
        }
    }

    pub fn medium(&self) -> media::Type {
        unsafe { media::Type::from((*self.as_ptr()).codec_type) }
    }

    pub fn set_medium(&mut self, value: media::Type) {
        unsafe {
            (*self.as_mut_ptr()).codec_type = value.into();
        }
    }

    pub fn id(&self) -> Id {
        unsafe { Id::from((*self.as_ptr()).codec_id) }
    }

    pub fn set_id(&mut self, value: Id) {
        unsafe {
            (*self.as_mut_ptr()).codec_id = value.into();
        }
    }

    pub fn bit_rate(&self) -> i64 {
        unsafe { (*self.as_ptr()).bit_rate }
    }

    pub fn set_bit_rate(&mut self, value: i64) {
        unsafe {
            (*self.as_mut_ptr()).bit_rate = value;
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Parameters {
    fn drop(&mut self) {
        unsafe {
            if self.owner.is_none() {
                avcodec_parameters_free(&mut self.as_mut_ptr());
            }
        }
    }
}

impl Clone for Parameters {
    fn clone(&self) -> Self {
        let mut ctx = Parameters::new();
        ctx.clone_from(self);

        ctx
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            avcodec_parameters_copy(self.as_mut_ptr(), source.as_ptr());
        }
    }
}

impl<C: AsRef<Context>> From<C> for Parameters {
    fn from(context: C) -> Parameters {
        let mut parameters = Parameters::new();
        let context = context.as_ref();
        unsafe {
            avcodec_parameters_from_context(parameters.as_mut_ptr(), context.as_ptr());
        }
        parameters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_codec_parameters() {
        let mut parameters = Parameters::new();
        parameters.set_medium(media::Type::Subtitle);
        parameters.set_id(Id::WEBVTT);
        parameters.set_bit_rate(64_000);

        assert_eq!(parameters.medium(), media::Type::Subtitle);
        assert_eq!(parameters.id(), Id::WEBVTT);
        assert_eq!(parameters.bit_rate(), 64_000);
    }
}
