use std::marker::PhantomData;
use std::mem;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;
use std::ops::Deref;

use libc::{c_int, c_uint, uint32_t, int64_t};
use ffi::*;
use ::format;
use ::Picture;

bitflags! {
	flags Flags: c_int {
		const FLAG_FORCED = AV_SUBTITLE_FLAG_FORCED,
	}
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Type {
	None,
	Bitmap,
	Text,
	Ass,
}

impl From<AVSubtitleType> for Type {
	fn from(value: AVSubtitleType) -> Type {
		match value {
			SUBTITLE_NONE   => Type::None,
			SUBTITLE_BITMAP => Type::Bitmap,
			SUBTITLE_TEXT   => Type::Text,
			SUBTITLE_ASS    => Type::Ass
		}
	}
}

impl Into<AVSubtitleType> for Type {
	fn into(self) -> AVSubtitleType {
		match self {
			Type::None   => SUBTITLE_NONE,
			Type::Bitmap => SUBTITLE_BITMAP,
			Type::Text   => SUBTITLE_TEXT,
			Type::Ass    => SUBTITLE_ASS
		}
	}
}

pub struct Subtitle {
	pub val: AVSubtitle,
}

impl Subtitle {
	pub fn new() -> Self {
		unsafe {
			Subtitle { val: mem::zeroed() }
		}
	}

	pub fn pts(&self) -> i64 {
		self.val.pts as i64
	}

	pub fn set_pts(&mut self, value: i64) {
		self.val.pts = value as int64_t;
	}

	pub fn start(&self) -> u32 {
		self.val.start_display_time as u32
	}

	pub fn set_start(&mut self, value: u32) {
		self.val.start_display_time = value as uint32_t;
	}

	pub fn end(&self) -> u32 {
		self.val.end_display_time as u32
	}

	pub fn set_end(&mut self, value: u32) {
		self.val.end_display_time = value as uint32_t;
	}

	pub fn rects(&self) -> RectIter {
		RectIter::new(&self.val)
	}
}

pub struct RectIter<'a> {
	ptr: *const AVSubtitle,
	cur: c_uint,

	_marker: PhantomData<&'a Subtitle>,
}

impl<'a> RectIter<'a> {
	pub fn new(ptr: *const AVSubtitle) -> Self {
		RectIter { ptr: ptr, cur: 0, _marker: PhantomData }
	}
}

impl<'a> Iterator for RectIter<'a> {
	type Item = Rect<'a>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			if self.cur >= (*self.ptr).num_rects {
				None
			}
			else {
				self.cur += 1;
				Some(Rect::wrap(*(*self.ptr).rects.offset((self.cur - 1) as isize)))
			}
		}
	}
}

pub enum Rect<'a> {
	None,
	Bitmap(Bitmap<'a>),
	Text(Text<'a>),
	Ass(Ass<'a>),
}

impl<'a> Rect<'a> {
	pub fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		unsafe {
			match Type::from((*ptr).kind) {
				Type::None   => Rect::None,
				Type::Bitmap => Rect::Bitmap(Bitmap::wrap(ptr)),
				Type::Text   => Rect::Text(Text::wrap(ptr)),
				Type::Ass    => Rect::Ass(Ass::wrap(ptr))
			}
		}
	}

	pub fn flags(&self) -> Flags {
		unsafe {
			Flags::from_bits_truncate(match self {
				&Rect::None          => 0,
				&Rect::Bitmap(ref b) => (*b.ptr).flags,
				&Rect::Text(ref t)   => (*t.ptr).flags,
				&Rect::Ass(ref a)    => (*a.ptr).flags
			})
		}
	}
}

pub struct Bitmap<'a> {
	pub ptr: *mut AVSubtitleRect,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Bitmap<'a> {
	pub fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		Bitmap { ptr: ptr, _marker: PhantomData }
	}

	pub fn x(&self) -> usize {
		unsafe {
			(*self.ptr).x as usize
		}
	}

	pub fn y(&self) -> usize {
		unsafe {
			(*self.ptr).y as usize
		}
	}

	pub fn width(&self) -> u32 {
		unsafe {
			(*self.ptr).w as u32
		}
	}

	pub fn height(&self) -> u32 {
		unsafe {
			(*self.ptr).h as u32
		}
	}

	pub fn colors(&self) -> usize {
		unsafe {
			(*self.ptr).nb_colors as usize
		}
	}

	pub fn picture(&self, format: format::Pixel) -> Picture<'a> {
		unsafe {
			Picture::wrap(&mut (*self.ptr).pict, format, (*self.ptr).w as u32, (*self.ptr).h as u32)
		}
	}
}

pub struct Text<'a> {
	pub ptr: *mut AVSubtitleRect,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Text<'a> {
	pub fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		Text { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Deref for Text<'a> {
	type Target = str;

	fn deref<'b>(&'b self) -> &'b str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).text).to_bytes())
		}
	}
}

pub struct Ass<'a> {
	pub ptr: *mut AVSubtitleRect,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Ass<'a> {
	pub fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		Ass { ptr: ptr, _marker: PhantomData }
	}
}

impl<'a> Deref for Ass<'a> {
	type Target = str;

	fn deref<'b>(&'b self) -> &'b str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).ass).to_bytes())
		}
	}
}
