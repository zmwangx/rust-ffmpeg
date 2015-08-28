extern crate ffmpeg;

use std::env;

fn main() {
	ffmpeg::init().unwrap();

	for arg in env::args().skip(1) {
		if let Some(codec) = ffmpeg::decoder::find_by_name(&arg) {
			println!("type: decoder");
			println!("\t id: {:?}", codec.id());
			println!("\t name: {}", codec.name());
			println!("\t description: {}", codec.description());
			println!("\t medium: {:?}", codec.medium());
			println!("\t capabilities: {:?}", codec.capabilities());

			if let Some(profiles) = codec.profiles() {
				println!("\t profiles: {:?}", profiles.collect::<Vec<_>>());
			}

			if let Ok(video) = codec.video() {
				println!("\t rates: {:?}", video.rates().collect::<Vec<_>>());
				println!("\t formats: {:?}", video.formats().collect::<Vec<_>>());
			}

			if let Ok(audio) = codec.audio() {
				println!("\t samplerates: {:?}", audio.rates().collect::<Vec<_>>());
				println!("\t sample_formats: {:?}", audio.formats().collect::<Vec<_>>());
				println!("\t channel_layouts: {:?}", audio.channel_layouts().collect::<Vec<_>>());
			}

			println!("\t max_lowres: {:?}", codec.max_lowres());
		}

		if let Some(codec) = ffmpeg::encoder::find_by_name(&arg) {
			println!("");
			println!("type: encoder");
			println!("\t id: {:?}", codec.id());
			println!("\t name: {}", codec.name());
			println!("\t description: {}", codec.description());
			println!("\t medium: {:?}", codec.medium());
			println!("\t capabilities: {:?}", codec.capabilities());

			if let Some(profiles) = codec.profiles() {
				println!("\t profiles: {:?}", profiles.collect::<Vec<_>>());
			}

			if let Ok(video) = codec.video() {
				println!("\t rates: {:?}", video.rates().collect::<Vec<_>>());
				println!("\t formats: {:?}", video.formats().collect::<Vec<_>>());
			}

			if let Ok(audio) = codec.audio() {
				println!("\t samplerates: {:?}", audio.rates().collect::<Vec<_>>());
				println!("\t sample_formats: {:?}", audio.formats().collect::<Vec<_>>());
				println!("\t channel_layouts: {:?}", audio.channel_layouts().collect::<Vec<_>>());
			}

			println!("\t max_lowres: {:?}", codec.max_lowres());
		}
	}
}
