pub mod flag;
pub use self::flag::Flags;

use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;
use std::ops::Deref;

use libc::{c_uint, uint32_t};
use ffi::*;
use ::format;
use ::Picture;

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

pub struct Subtitle(AVSubtitle);

impl Subtitle {
	pub unsafe fn as_ptr(&self) -> *const AVSubtitle {
		&self.0
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVSubtitle {
		&mut self.0
	}
}

impl Subtitle {
	pub fn new() -> Self {
		unsafe {
			Subtitle(mem::zeroed())
		}
	}

	pub fn pts(&self) -> Option<i64> {
		match self.0.pts {
			AV_NOPTS_VALUE => None,
			pts            => Some(pts)
		}
	}

	pub fn set_pts(&mut self, value: Option<i64>) -> &mut Self {
		self.0.pts = value.unwrap_or(AV_NOPTS_VALUE);

		self
	}

	pub fn start(&self) -> u32 {
		self.0.start_display_time as u32
	}

	pub fn set_start(&mut self, value: u32) -> &mut Self {
		self.0.start_display_time = value as uint32_t;

		self
	}

	pub fn end(&self) -> u32 {
		self.0.end_display_time as u32
	}

	pub fn set_end(&mut self, value: u32) -> &mut Self {
		self.0.end_display_time = value as uint32_t;

		self
	}

	pub fn rects(&self) -> RectIter {
		RectIter::new(&self.0)
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

	fn size_hint(&self) -> (usize, Option<usize>) {
		unsafe {
			((*self.ptr).num_rects as usize, Some((*self.ptr).num_rects as usize))
		}
	}
}

impl<'a> ExactSizeIterator for RectIter<'a> { }

pub enum Rect<'a> {
	None,
	Bitmap(Bitmap<'a>),
	Text(Text<'a>),
	Ass(Ass<'a>),
}

impl<'a> Rect<'a> {
	pub unsafe fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		match Type::from((*ptr).kind) {
			Type::None   => Rect::None,
			Type::Bitmap => Rect::Bitmap(Bitmap::wrap(ptr)),
			Type::Text   => Rect::Text(Text::wrap(ptr)),
			Type::Ass    => Rect::Ass(Ass::wrap(ptr))
		}
	}

	pub unsafe fn as_ptr(&self) -> *const AVSubtitleRect {
		match self {
			&Rect::None          => ptr::null(),
			&Rect::Bitmap(ref b) => b.as_ptr(),
			&Rect::Text(ref t)   => t.as_ptr(),
			&Rect::Ass(ref a)    => a.as_ptr()
		}
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVSubtitleRect {
		match self {
			&mut Rect::None          => ptr::null_mut(),
			&mut Rect::Bitmap(ref mut b) => b.as_mut_ptr(),
			&mut Rect::Text(ref mut t)   => t.as_mut_ptr(),
			&mut Rect::Ass(ref mut a)    => a.as_mut_ptr()
		}
	}
}

impl<'a> Rect<'a> {
	pub fn flags(&self) -> Flags {
		unsafe {
			Flags::from_bits_truncate(match self {
				&Rect::None          => 0,
				&Rect::Bitmap(ref b) => (*b.as_ptr()).flags,
				&Rect::Text(ref t)   => (*t.as_ptr()).flags,
				&Rect::Ass(ref a)    => (*a.as_ptr()).flags
			})
		}
	}
}

pub struct Bitmap<'a> {
	ptr: *mut AVSubtitleRect,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Bitmap<'a> {
	pub unsafe fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		Bitmap { ptr: ptr, _marker: PhantomData }
	}

	pub unsafe fn as_ptr(&self) -> *const AVSubtitleRect {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVSubtitleRect {
		self.ptr
	}
}

impl<'a> Bitmap<'a> {
	pub fn x(&self) -> usize {
		unsafe {
			(*self.as_ptr()).x as usize
		}
	}

	pub fn y(&self) -> usize {
		unsafe {
			(*self.as_ptr()).y as usize
		}
	}

	pub fn width(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).w as u32
		}
	}

	pub fn height(&self) -> u32 {
		unsafe {
			(*self.as_ptr()).h as u32
		}
	}

	pub fn colors(&self) -> usize {
		unsafe {
			(*self.as_ptr()).nb_colors as usize
		}
	}

	// XXX: verify safety
	pub fn picture(&self, format: format::Pixel) -> Picture<'a> {
		unsafe {
			Picture::wrap(&(*self.as_ptr()).pict as *const _ as *mut _, format, (*self.as_ptr()).w as u32, (*self.as_ptr()).h as u32)
		}
	}
}

pub struct Text<'a> {
	ptr: *mut AVSubtitleRect,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Text<'a> {
	pub unsafe fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		Text { ptr: ptr, _marker: PhantomData }
	}

	pub unsafe fn as_ptr(&self) -> *const AVSubtitleRect {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVSubtitleRect {
		self.ptr
	}
}

impl<'a> Deref for Text<'a> {
	type Target = str;

	fn deref<'b>(&'b self) -> &'b str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).text).to_bytes())
		}
	}
}

pub struct Ass<'a> {
	ptr: *mut AVSubtitleRect,

	_marker: PhantomData<&'a ()>,
}

impl<'a> Ass<'a> {
	pub unsafe fn wrap(ptr: *mut AVSubtitleRect) -> Self {
		Ass { ptr: ptr, _marker: PhantomData }
	}

	pub unsafe fn as_ptr(&self) -> *const AVSubtitleRect {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVSubtitleRect {
		self.ptr
	}
}

impl<'a> Deref for Ass<'a> {
	type Target = str;

	fn deref<'b>(&'b self) -> &'b str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).ass).to_bytes())
		}
	}
}
