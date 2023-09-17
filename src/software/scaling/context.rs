use std::ptr;

use super::Flags;
use ffi::*;
use libc::c_int;
use util::format;
use {frame, Error};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Definition {
    pub format: format::Pixel,
    pub width: u32,
    pub height: u32,
}

pub struct Context {
    ptr: *mut SwsContext,

    input: Definition,
    output: Definition,
}

impl Context {
    #[inline(always)]
    pub unsafe fn as_ptr(&self) -> *const SwsContext {
        self.ptr as *const _
    }

    #[inline(always)]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut SwsContext {
        self.ptr
    }
}

impl Context {
    pub fn get(
        src_format: format::Pixel,
        src_w: u32,
        src_h: u32,
        dst_format: format::Pixel,
        dst_w: u32,
        dst_h: u32,
        flags: Flags,
    ) -> Result<Self, Error> {
        unsafe {
            let ptr = sws_getContext(
                src_w as c_int,
                src_h as c_int,
                src_format.into(),
                dst_w as c_int,
                dst_h as c_int,
                dst_format.into(),
                flags.bits(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if !ptr.is_null() {
                Ok(Context {
                    ptr,

                    input: Definition {
                        format: src_format,
                        width: src_w,
                        height: src_h,
                    },

                    output: Definition {
                        format: dst_format,
                        width: dst_w,
                        height: dst_h,
                    },
                })
            } else {
                Err(Error::InvalidData)
            }
        }
    }

    pub fn cached(
        &mut self,
        src_format: format::Pixel,
        src_w: u32,
        src_h: u32,
        dst_format: format::Pixel,
        dst_w: u32,
        dst_h: u32,
        flags: Flags,
    ) {
        self.input = Definition {
            format: src_format,
            width: src_w,
            height: src_h,
        };

        self.output = Definition {
            format: dst_format,
            width: dst_w,
            height: dst_h,
        };

        unsafe {
            self.ptr = sws_getCachedContext(
                self.as_mut_ptr(),
                src_w as c_int,
                src_h as c_int,
                src_format.into(),
                dst_w as c_int,
                dst_h as c_int,
                dst_format.into(),
                flags.bits(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            );
        }
    }

    #[inline]
    pub fn input(&self) -> &Definition {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &Definition {
        &self.output
    }

    pub fn run(&mut self, input: &frame::Video, output: &mut frame::Video) -> Result<(), Error> {
        if input.format() != self.input.format
            || input.width() != self.input.width
            || input.height() != self.input.height
        {
            return Err(Error::InputChanged);
        }

        unsafe {
            if output.is_empty() {
                output.alloc(self.output.format, self.output.width, self.output.height);
            }
        }

        if output.format() != self.output.format
            || output.width() != self.output.width
            || output.height() != self.output.height
        {
            return Err(Error::OutputChanged);
        }

        unsafe {
            sws_scale(
                self.as_mut_ptr(),
                (*input.as_ptr()).data.as_ptr() as *const *const _,
                (*input.as_ptr()).linesize.as_ptr() as *const _,
                0,
                self.input.height as c_int,
                (*output.as_mut_ptr()).data.as_ptr(),
                (*output.as_mut_ptr()).linesize.as_ptr() as *mut _,
            );
        }

        Ok(())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sws_freeContext(self.as_mut_ptr());
        }
    }
}
