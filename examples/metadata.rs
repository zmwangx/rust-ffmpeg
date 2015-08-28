extern crate ffmpeg;

use std::env;
use std::path::Path;

fn print_metadata(context: &ffmpeg::format::Context) {
	for (k, v) in context.metadata().iter() {
		println!("{}: {}", k, v);
	}

	if let Some(stream) = context.streams().best(ffmpeg::media::Type::Video) {
		println!("Best video stream index: {}", stream.index());
	}

	if let Some(stream) = context.streams().best(ffmpeg::media::Type::Audio) {
		println!("Best audio stream index: {}", stream.index());
	}

	if let Some(stream) = context.streams().best(ffmpeg::media::Type::Subtitle) {
		println!("Best subtitle stream index: {}", stream.index());
	}

	for stream in context.streams() {
		println!("stream index {}:", stream.index());
		println!("\ttime_base: {}", stream.time_base());
		println!("\tstart_time: {}", stream.start_time());
		println!("\tduration: {}", stream.duration());
		println!("\tframes: {}", stream.frames());
		println!("\tdisposition: {:?}", stream.disposition());
		println!("\tdiscard: {:?}", stream.discard());
		println!("\tframe_rate: {}", stream.frame_rate());

		let codec = stream.codec();
		println!("\tmedium: {:?}", codec.medium());
		println!("\tid: {:?}", codec.id());
		println!("\tbit_rate: {}", codec.bit_rate());
		println!("\tdelay: {}", codec.delay());

		if let Ok(decoder) = codec.decoder() {
			if let Ok(video) = decoder.video() {
				println!("\tvideo.width: {}", video.width());
				println!("\tvideo.height: {}", video.height());
				println!("\tvideo.format: {:?}", video.format());
				println!("\tvideo.has_b_frames: {}", video.has_b_frames());
				println!("\tvideo.aspect_ratio: {}", video.aspect_ratio());
				println!("\tvideo.color_space: {:?}", video.color_space());
				println!("\tvideo.color_range: {:?}", video.color_range());
				println!("\tvideo.color_primaries: {:?}", video.color_primaries());
				println!("\tvideo.color_transfer_characteristic: {:?}", video.color_transfer_characteristic());
				println!("\tvideo.chroma_location: {:?}", video.chroma_location());
				println!("\tvideo.references: {}", video.references());
				println!("\tvideo.intra_dc_precision: {}", video.intra_dc_precision());
			}
		}

		if let Ok(decoder) = codec.decoder() {
			if let Ok(audio) = decoder.audio() {
				println!("\taudio.rate: {}", audio.rate());
				println!("\taudio.channels: {}", audio.channels());
				println!("\taudio.format: {:?}", audio.format());
				println!("\taudio.frames: {}", audio.frames());
				println!("\taudio.align: {}", audio.align());
				println!("\taudio.channel_layout: {:?}", audio.channel_layout());
				println!("\taudio.max_rate: {}", audio.max_rate());
				println!("\taudio.frame_start: {:?}", audio.frame_start());
			}
		}
	}
}

fn main() {
	ffmpeg::init().unwrap();

	if let Some(file) = env::args().nth(1) {
		let path = Path::new(&file);

		match ffmpeg::format::open(&path) {
			Ok(context) => print_metadata(&context),
			Err(error)  => println!("error: {}", error),
		}
	}
	else {
		println!("Usage: metadata <path>")
	}
}
