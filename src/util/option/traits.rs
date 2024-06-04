//! NOTE: this will be much better once specialization comes

use std::ffi::CString;
use std::mem;

use ffi::*;
use libc::c_int;
use util::format;
use {ChannelLayout, Error, Rational};

macro_rules! check {
    ($expr:expr) => {
        match $expr {
            0 => Ok(()),
            e => Err(Error::from(e)),
        }
    };
}

pub unsafe trait Target<T> {
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;
}

pub trait Settable<T>: Target<T> {
    fn set<V: 'static>(&mut self, name: &str, value: &V) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_bin(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                value as *const _ as *const _,
                mem::size_of::<V>() as c_int,
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_str(&mut self, name: &str, value: &str) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();
            let value = CString::new(value).unwrap();

            check!(av_opt_set(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                value.as_ptr(),
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_int(&mut self, name: &str, value: i64) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_int(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                value,
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_double(&mut self, name: &str, value: f64) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_double(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                value,
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_rational<V: Into<Rational>>(&mut self, name: &str, value: V) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_q(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                value.into().into(),
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_image_size(&mut self, name: &str, w: u32, h: u32) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_image_size(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                w as c_int,
                h as c_int,
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_pixel_format(&mut self, name: &str, format: format::Pixel) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_pixel_fmt(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                format.into(),
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_sample_format(&mut self, name: &str, format: format::Sample) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            check!(av_opt_set_sample_fmt(
                self.as_mut_ptr() as *mut _,
                name.as_ptr(),
                format.into(),
                AV_OPT_SEARCH_CHILDREN
            ))
        }
    }

    fn set_channel_layout(&mut self, name: &str, layout: ChannelLayout) -> Result<(), Error> {
        unsafe {
            let name = CString::new(name).unwrap();

            #[cfg(not(feature = "ffmpeg_7_0"))]
            {
                check!(av_opt_set_channel_layout(
                    self.as_mut_ptr() as *mut _,
                    name.as_ptr(),
                    layout.bits() as i64,
                    AV_OPT_SEARCH_CHILDREN
                ))
            }

            #[cfg(feature = "ffmpeg_7_0")]
            {
                check!(av_opt_set_chlayout(
                    self.as_mut_ptr() as *mut _,
                    name.as_ptr(),
                    &layout.into(),
                    AV_OPT_SEARCH_CHILDREN
                ))
            }
        }
    }
}

pub trait Gettable<T>: Target<T> {}

pub trait Iterable<T>: Target<T> {}
