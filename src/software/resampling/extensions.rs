use util::format;
use ::{decoder, Error, ChannelLayout};
use super::Context;

impl decoder::Audio {
	pub fn resampler(&self, format: format::Sample, channel_layout: ChannelLayout, rate: u32) -> Result<Context, Error> {
		Context::get(self.format(), self.channel_layout(), self.rate(),
		             format, channel_layout, rate)
	}
}
