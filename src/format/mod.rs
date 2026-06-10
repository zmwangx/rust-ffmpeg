pub use util::format::{pixel, Pixel};
pub use util::format::{sample, Sample};
use util::interrupt;

pub mod stream;

pub mod chapter;

pub mod context;
pub use self::context::Context;

pub mod format;
#[cfg(not(feature = "ffmpeg_5_0"))]
pub use self::format::list;
pub use self::format::{flag, Flags};
pub use self::format::{Input, Output};

pub mod network;

use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;
use std::str::from_utf8_unchecked;

use ffi::*;
use {Dictionary, Error, Format};

#[cfg(not(feature = "ffmpeg_5_0"))]
pub fn register_all() {
    unsafe {
        av_register_all();
    }
}

#[cfg(not(feature = "ffmpeg_5_0"))]
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

// XXX: use to_cstring when stable
fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> CString {
    CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap()
}

fn opt_cstring(s: Option<&str>) -> Result<Option<CString>, Error> {
    s.map(CString::new)
        .transpose()
        .map_err(|_| Error::Other { errno: EINVAL })
}

// NOTE: this will be better with specialization or anonymous return types
pub fn open<P: AsRef<Path> + ?Sized>(path: &P, format: &Format) -> Result<Context, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);

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

pub fn open_with<P: AsRef<Path> + ?Sized>(
    path: &P,
    format: &Format,
    options: Dictionary,
) -> Result<Context, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);
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

pub fn input<P: AsRef<Path> + ?Sized>(path: &P) -> Result<context::Input, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);

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

pub fn input_with_dictionary<P: AsRef<Path> + ?Sized>(
    path: &P,
    options: Dictionary,
) -> Result<context::Input, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);
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

pub fn input_with_interrupt<P: AsRef<Path> + ?Sized, F>(
    path: &P,
    closure: F,
) -> Result<context::Input, Error>
where
    F: FnMut() -> bool + 'static,
{
    unsafe {
        let mut ps = avformat_alloc_context();
        if ps.is_null() {
            return Err(Error::Other { errno: ENOMEM });
        }
        let path = from_path(path);
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

pub fn input_with_interrupt_and_dictionary<F>(
    path: &Path,
    closure: F,
    options: Dictionary,
) -> Result<context::Input, Error>
where
    F: FnMut() -> bool + 'static,
{
    unsafe {
        let mut ps = avformat_alloc_context();
        (*ps).interrupt_callback = interrupt::new(Box::new(closure)).interrupt;
        let path = from_path(path);

        let mut opts = options.disown();
        let res = avformat_open_input(&raw mut ps, path.as_ptr(), ptr::null_mut(), &raw mut opts);
        Dictionary::own(opts);

        match res {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&raw mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}
/// Opens an input from a readable `context::StreamIo` (created with
/// `StreamIo::from_read` or `StreamIo::from_read_seek`).
///
/// An optional filename helps with format detection; options configure the
/// format context. Fails with `EINVAL` if `custom_io` is a write context or
/// `filename` contains an interior NUL byte.
pub fn input_from_stream(
    mut custom_io: context::StreamIo,
    filename: Option<&str>,
    options: Option<Dictionary>,
) -> Result<context::Input, Error> {
    if custom_io.is_writable() {
        return Err(Error::Other { errno: EINVAL });
    }

    let filename = opt_cstring(filename)?;
    let filename_ptr = filename.as_ref().map_or(ptr::null(), |f| f.as_ptr());

    unsafe {
        let mut ps = avformat_alloc_context();
        if ps.is_null() {
            return Err(Error::Other { errno: ENOMEM });
        }
        (*ps).pb = custom_io.as_mut_ptr();
        (*ps).flags |= AVFMT_FLAG_CUSTOM_IO;

        let result = if let Some(opts) = options {
            let mut opts = opts.disown();
            let res = avformat_open_input(&mut ps, filename_ptr, ptr::null_mut(), &mut opts);
            Dictionary::own(opts);
            res
        } else {
            avformat_open_input(&mut ps, filename_ptr, ptr::null_mut(), ptr::null_mut())
        };

        match result {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap_with_custom_io(ps, custom_io)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn output<P: AsRef<Path> + ?Sized>(path: &P) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);

        match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
            0 => match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
                0 => Ok(context::Output::wrap(ps)),
                e => Err(Error::from(e)),
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn output_with<P: AsRef<Path> + ?Sized>(
    path: &P,
    options: Dictionary,
) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);
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

pub fn output_as<P: AsRef<Path> + ?Sized>(
    path: &P,
    format: &str,
) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);
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

pub fn output_as_with<P: AsRef<Path> + ?Sized>(
    path: &P,
    format: &str,
    options: Dictionary,
) -> Result<context::Output, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_path(path);
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

/// Creates an output context that writes to a writable `context::StreamIo`
/// (created with `StreamIo::from_write` or `StreamIo::from_write_seek`).
///
/// The output format is inferred from `filename` or given explicitly via
/// `format`; most muxers need a seekable stream for well-formed output. Call
/// `write_trailer` before dropping the returned context — dropping only
/// flushes what the muxer already emitted, it cannot finalize the file.
/// Fails with `EINVAL` if `custom_io` is not a write context, if `filename` /
/// `format` contain an interior NUL byte, or if the resolved muxer does its
/// own I/O and would never write to the stream (`AVFMT_NOFILE` formats like
/// `image2` or output devices).
pub fn output_to_stream(
    mut custom_io: context::StreamIo,
    filename: Option<&str>,
    format: Option<&str>,
) -> Result<context::Output, Error> {
    if !custom_io.is_writable() {
        return Err(Error::Other { errno: EINVAL });
    }

    let filename = opt_cstring(filename)?;
    let filename_ptr = filename.as_ref().map_or(ptr::null(), |f| f.as_ptr());

    let format = opt_cstring(format)?;
    let format_ptr = format.as_ref().map_or(ptr::null(), |f| f.as_ptr());

    unsafe {
        let mut ps = ptr::null_mut();

        match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), format_ptr, filename_ptr) {
            0 => {
                // AVFMT_NOFILE muxers (image2's one-file-per-frame, devices,
                // ...) do their own I/O, and `AVFormatContext.pb` is
                // documented to stay NULL for them; the caller's stream would
                // silently never receive the muxed output.
                if (*(*ps).oformat).flags & AVFMT_NOFILE != 0 {
                    avformat_free_context(ps);
                    return Err(Error::Other { errno: EINVAL });
                }

                (*ps).pb = custom_io.as_mut_ptr();
                (*ps).flags |= AVFMT_FLAG_CUSTOM_IO;

                Ok(context::Output::wrap_with_custom_io(ps, custom_io))
            }

            e => Err(Error::from(e)),
        }
    }
}
