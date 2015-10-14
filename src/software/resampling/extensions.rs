use util::format;
use ::{decoder, Error, ChannelLayout, frame};
use super::Context;

impl frame::Audio {
	#[inline]
	pub fn resampler(&self, format: format::Sample, channel_layout: ChannelLayout, rate: u32) -> Result<Context, Error> {
		Context::get(self.format(), self.channel_layout(), self.rate(),
		             format, channel_layout, rate)
	}
}

impl decoder::Audio {
	#[inline]
	pub fn resampler(&self, format: format::Sample, channel_layout: ChannelLayout, rate: u32) -> Result<Context, Error> {
		Context::get(self.format(), self.channel_layout(), self.rate(),
		             format, channel_layout, rate)
	}
}
