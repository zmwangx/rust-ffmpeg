#![feature(convert)]
#![allow(raw_pointer_derive, non_camel_case_types)]

extern crate libc;
extern crate ffmpeg_sys as ffi;
#[macro_use] extern crate bitflags;

pub mod util;
pub use util::error::Error;
pub use util::dictionary::Dictionary;
pub use util::rational::Rational;
pub use util::media;
pub use util::picture;
pub use util::color;
pub use util::chroma;
pub use util::frame::{self, Frame};

#[cfg(feature = "format")]
pub mod format;
#[cfg(feature = "format")]
pub use format::format::Format;
#[cfg(feature = "format")]
pub use format::stream::Stream;

#[cfg(feature = "codec")]
pub mod codec;
#[cfg(feature = "codec")]
pub use codec::packet::{self, Packet};
#[cfg(feature = "codec")]
pub use codec::subtitle::{self, Subtitle};
#[cfg(feature = "codec")]
pub use codec::picture::Picture;
#[cfg(feature = "codec")]
pub use codec::discard::Discard;
#[cfg(feature = "codec")]
pub use codec::codec::Codec;
#[cfg(feature = "codec")]
pub use codec::field_order::FieldOrder;
#[cfg(feature = "codec")]
pub use codec::audio_service::AudioService;

#[cfg(feature = "device")]
pub mod device;
