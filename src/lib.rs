#![feature(convert)]
#![allow(raw_pointer_derive, non_camel_case_types)]

extern crate libc;
extern crate ffmpeg_sys as ffi;
#[macro_use] extern crate bitflags;

pub mod util;
pub use util::error::Error;
pub use util::dictionary::Dictionary;
pub use util::rational::Rational;
pub use util::color_space::ColorSpace;
pub use util::color_range::ColorRange;
pub use util::media;
pub use util::sample_format::SampleFormat;
pub use util::pixel_format::PixelFormat;
