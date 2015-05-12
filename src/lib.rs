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
pub use util::frame::{self, Frame};

pub mod format;

pub mod codec;
pub use codec::packet::{self, Packet};
pub use codec::subtitle::Subtitle;
pub use codec::discard::Discard;
pub use codec::codec::Codec;
pub use codec::encoder::{self, Encode};
pub use codec::decoder::{self, Decode};
