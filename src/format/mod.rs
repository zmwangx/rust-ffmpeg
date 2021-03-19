pub use util::format::{pixel, Pixel};
pub use util::format::{sample, Sample};
use util::interrupt;

pub mod stream;

pub mod chapter;

pub mod context;
pub use self::context::Context;

pub mod format;
pub use self::format::{flag, Flags};
pub use self::format::{list, Input, Output};

pub mod network;

use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;
use std::str::from_utf8_unchecked;

use ffi::*;
use {Dictionary, Error, Format};

pub fn register_all() {
    unsafe {
        av_register_all();
    }
}

pub fn register(format: &Format) {
    match *format {
        Format::Input(ref format) => unsafe {
            av_register_input_format(format.as_ptr() as *mut _);
        },

        Format::Output(ref format) => unsafe {
            av_register_output_format(format.as_ptr() as *mut _);
        },
    }
}

pub fn version() -> u32 {
    unsafe { avformat_version() }
}

pub fn configuration() -> &'static str {
    unsafe { from_utf8_unchecked(CStr::from_ptr(avformat_configuration()).to_bytes()) }
}

pub fn license() -> &'static str {
    unsafe { from_utf8_unchecked(CStr::from_ptr(avformat_license()).to_bytes()) }
}

pub fn open<P: AsRef<OsStr>>(path_or_url: P, format: &Format) -> Result<Context, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);

        match *format {
            Format::Input(ref format) => match avformat_open_input(
                &mut ps,
                path.as_ptr(),
                format.as_ptr() as *mut _,
                ptr::null_mut(),
            ) {
                0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                    r if r >= 0 => Ok(Context::Input(context::Input::wrap(ps))),
                    e => Err(Error::from(e)),
                },

                e => Err(Error::from(e)),
            },

            Format::Output(ref format) => match avformat_alloc_output_context2(
                &mut ps,
                format.as_ptr() as *mut _,
                ptr::null(),
                path.as_ptr(),
            ) {
                0 => match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
                    0 => Ok(Context::Output(context::Output::wrap(ps))),
                    e => Err(Error::from(e)),
                },

                e => Err(Error::from(e)),
            },
        }
    }
}

pub fn open_with<P: AsRef<OsStr>>(
    path_or_url: P,
    format: &Format,
    options: Dictionary,
) -> Result<Context, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);
        let mut opts = options.disown();

        match *format {
            Format::Input(ref format) => {
                let res = avformat_open_input(
                    &mut ps,
                    path.as_ptr(),
                    format.as_ptr() as *mut _,
                    &mut opts,
                );

                Dictionary::own(opts);

                match res {
                    0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                        r if r >= 0 => Ok(Context::Input(context::Input::wrap(ps))),
                        e => Err(Error::from(e)),
                    },

                    e => Err(Error::from(e)),
                }
            }

            Format::Output(ref format) => match avformat_alloc_output_context2(
                &mut ps,
                format.as_ptr() as *mut _,
                ptr::null(),
                path.as_ptr(),
            ) {
                0 => match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
                    0 => Ok(Context::Output(context::Output::wrap(ps))),
                    e => Err(Error::from(e)),
                },

                e => Err(Error::from(e)),
            },
        }
    }
}

pub fn input<P: AsRef<OsStr>>(path_or_url: P) -> Result<context::Input, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);

        match avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), ptr::null_mut()) {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn input_with_dictionary<P: AsRef<OsStr>>(
    path_or_url: P,
    options: Dictionary,
) -> Result<context::Input, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);
        let mut opts = options.disown();
        let res = avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), &mut opts);

        Dictionary::own(opts);

        match res {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn input_with_interrupt<P: AsRef<OsStr>>(
    path_or_url: P,
    closure: impl FnMut() -> bool,
) -> Result<context::Input, Error> {
    unsafe {
        let mut ps = avformat_alloc_context();
        let path = from_os_str(path_or_url);
        (*ps).interrupt_callback = interrupt::new(Box::new(closure)).interrupt;

        match avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), ptr::null_mut()) {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn output<P: AsRef<OsStr>>(path_or_url: P) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);

        match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
            0 => match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
                0 => Ok(context::Output::wrap(ps)),
                e => Err(Error::from(e)),
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn output_with<P: AsRef<OsStr>>(
    path_or_url: P,
    options: Dictionary,
) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);
        let mut opts = options.disown();

        match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
            0 => {
                let res = avio_open2(
                    &mut (*ps).pb,
                    path.as_ptr(),
                    AVIO_FLAG_WRITE,
                    ptr::null(),
                    &mut opts,
                );

                Dictionary::own(opts);

                match res {
                    0 => Ok(context::Output::wrap(ps)),
                    e => Err(Error::from(e)),
                }
            }

            e => Err(Error::from(e)),
        }
    }
}

pub fn output_as<P: AsRef<OsStr>>(path_or_url: P, format: &str) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);
        let format = CString::new(format).unwrap();

        match avformat_alloc_output_context2(
            &mut ps,
            ptr::null_mut(),
            format.as_ptr(),
            path.as_ptr(),
        ) {
            0 => match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
                0 => Ok(context::Output::wrap(ps)),
                e => Err(Error::from(e)),
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn output_as_with<P: AsRef<OsStr>>(
    path_or_url: P,
    format: &str,
    options: Dictionary,
) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);
        let format = CString::new(format).unwrap();
        let mut opts = options.disown();

        match avformat_alloc_output_context2(
            &mut ps,
            ptr::null_mut(),
            format.as_ptr(),
            path.as_ptr(),
        ) {
            0 => {
                let res = avio_open2(
                    &mut (*ps).pb,
                    path.as_ptr(),
                    AVIO_FLAG_WRITE,
                    ptr::null(),
                    &mut opts,
                );

                Dictionary::own(opts);

                match res {
                    0 => Ok(context::Output::wrap(ps)),
                    e => Err(Error::from(e)),
                }
            }

            e => Err(Error::from(e)),
        }
    }
}

#[cfg(unix)]
fn from_os_str(path_or_url: impl AsRef<OsStr>) -> CString {
    use std::os::unix::ffi::OsStrExt;

    CString::new(path_or_url.as_ref().as_bytes()).unwrap()
}

#[cfg(not(unix))]
fn from_os_str(path_or_url: impl AsRef<OsStr>) -> CString {
    CString::new(path_or_url.as_ref().to_str().unwrap()).unwrap()
}
