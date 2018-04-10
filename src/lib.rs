#![allow(non_camel_case_types)]
#![cfg_attr(feature = "cargo-clippy", allow(inline_always))]

#[macro_use]
extern crate bitflags;
pub extern crate ffmpeg_sys as sys;
#[cfg(feature = "image")]
extern crate image;
extern crate libc;

pub use sys as ffi;

#[macro_use]
pub mod util;
pub use util::channel_layout::{self, ChannelLayout};
pub use util::chroma;
pub use util::color;
pub use util::dictionary;
pub use util::dictionary::Mut as DictionaryMut;
pub use util::dictionary::Owned as Dictionary;
pub use util::dictionary::Ref as DictionaryRef;
pub use util::error::Error;
pub use util::frame::{self, Frame};
pub use util::mathematics::{self, rescale, Rescale, Rounding};
pub use util::media;
pub use util::option;
pub use util::picture;
pub use util::rational::{self, Rational};
pub use util::time;

#[cfg(feature = "format")]
pub mod format;
#[cfg(feature = "format")]
pub use format::chapter::{Chapter, ChapterMut};
#[cfg(feature = "format")]
pub use format::format::Format;
#[cfg(feature = "format")]
pub use format::stream::{Stream, StreamMut};

#[cfg(feature = "codec")]
pub mod codec;
#[cfg(feature = "codec")]
pub use codec::audio_service::AudioService;
#[cfg(feature = "codec")]
pub use codec::codec::Codec;
#[cfg(feature = "codec")]
pub use codec::discard::Discard;
#[cfg(feature = "codec")]
pub use codec::field_order::FieldOrder;
#[cfg(feature = "codec")]
pub use codec::packet::{self, Packet};
#[cfg(feature = "codec")]
pub use codec::picture::Picture;
#[cfg(feature = "codec")]
pub use codec::subtitle::{self, Subtitle};
#[cfg(feature = "codec")]
pub use codec::threading;
#[cfg(feature = "codec")]
pub use codec::{decoder, encoder};

#[cfg(feature = "device")]
pub mod device;

#[cfg(feature = "filter")]
pub mod filter;
#[cfg(feature = "filter")]
pub use filter::Filter;

pub mod software;

fn init_error() {
    util::error::register_all();
}

#[cfg(feature = "format")]
fn init_format() {
    format::register_all();
}

#[cfg(not(feature = "format"))]
fn init_format() {}

#[cfg(feature = "device")]
fn init_device() {
    device::register_all();
}

#[cfg(not(feature = "device"))]
fn init_device() {}

#[cfg(feature = "filter")]
fn init_filter() {
    filter::register_all();
}

#[cfg(not(feature = "filter"))]
fn init_filter() {}

pub fn init() -> Result<(), Error> {
    init_error();
    init_format();
    init_device();
    init_filter();

    Ok(())
}
