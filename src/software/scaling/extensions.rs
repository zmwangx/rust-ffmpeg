use util::format;
use ::{Picture, decoder, Error, frame};
use super::{Context, Flags, flag};

impl<'a> Picture<'a> {
	#[inline]
	pub fn scaler(&self, width: u32, height: u32, flags: Flags) -> Result<Context, Error> {
		Context::get(self.format(), self.width(), self.height(),
		             self.format(), width, height,
		             flags)
	}

	#[inline]
	pub fn converter(&self, format: format::Pixel) -> Result<Context, Error> {
		Context::get(self.format(), self.width(), self.height(),
		             format, self.width(), self.height(),
		             flag::FAST_BILINEAR)
	}
}

impl frame::Video {
	#[inline]
	pub fn scaler(&self, width: u32, height: u32, flags: Flags) -> Result<Context, Error> {
		Context::get(self.format(), self.width(), self.height(),
		             self.format(), width, height,
		             flags)
	}

	#[inline]
	pub fn converter(&self, format: format::Pixel) -> Result<Context, Error> {
		Context::get(self.format(), self.width(), self.height(),
		             format, self.width(), self.height(),
		             flag::FAST_BILINEAR)
	}
}


impl decoder::Video {
	#[inline]
	pub fn scaler(&self, width: u32, height: u32, flags: Flags) -> Result<Context, Error> {
		Context::get(self.format(), self.width(), self.height(),
		             self.format(), width, height,
		             flags)
	}

	#[inline]
	pub fn converter(&self, format: format::Pixel) -> Result<Context, Error> {
		Context::get(self.format(), self.width(), self.height(),
		             format, self.width(), self.height(),
		             flag::FAST_BILINEAR)
	}
}
