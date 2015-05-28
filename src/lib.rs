#![feature(convert)]
#![allow(raw_pointer_derive, non_camel_case_types)]

extern crate libc;
extern crate ffmpeg_sys as ffi;
#[macro_use] extern crate bitflags;

pub mod util;
pub use util::error::Error;
pub use util::dictionary::Dictionary;
pub use util::rational::{self, Rational};
pub use util::media;
pub use util::picture;
pub use util::color;
pub use util::chroma;
pub use util::time;
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
pub use codec::{decoder, encoder};
#[cfg(feature = "codec")]
pub use codec::field_order::FieldOrder;
#[cfg(feature = "codec")]
pub use codec::audio_service::AudioService;
#[cfg(feature = "codec")]
pub use codec::threading;

#[cfg(feature = "device")]
pub mod device;

pub mod software;

fn init_error() {
	util::error::register_all();
}

#[cfg(feature = "format")]
fn init_format() {
	format::register_all();
}

#[cfg(not(feature = "format"))]
fn init_format() { }

#[cfg(feature = "device")]
fn init_device() {
	device::register_all();
}

#[cfg(not(feature = "device"))]
fn init_device() { }

pub fn init() -> Result<(), Error> {
	init_error();
	init_format();
	init_device();

	Ok(())
}
