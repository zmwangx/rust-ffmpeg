extern crate ffmpeg;

use std::env;
use std::path::Path;

use ffmpeg::{format, codec, frame, media, filter};
use ffmpeg::option::Settable;
use ffmpeg::{rescale, Rescale};

fn filter(spec: &str, decoder: &codec::decoder::Audio, encoder: &codec::encoder::Audio) -> Result<filter::Graph, ffmpeg::Error> {
	let mut filter = filter::Graph::new();

	let args = format!("time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
		decoder.time_base(), decoder.rate(), decoder.format().name(), decoder.channel_layout().bits());

	try!(filter.add(&filter::find("abuffer").unwrap(), "in", &args));
	try!(filter.add(&filter::find("abuffersink").unwrap(), "out", ""));

	{
		let mut out = filter.get("out").unwrap();

		out.set_sample_format(encoder.format());
		out.set_channel_layout(encoder.channel_layout());
		out.set_sample_rate(encoder.rate());
	}

	try!(try!(try!(filter.output("in", 0)).input("out", 0)).parse(spec));
	try!(filter.validate());

	println!("{}", filter.dump());

	if let Some(codec) = encoder.codec() {
		if !codec.capabilities().contains(ffmpeg::codec::capabilities::VARIABLE_FRAME_SIZE) {
			filter.get("out").unwrap().sink().set_frame_size(encoder.frame_size());
		}
	}

	Ok(filter)
}

struct Transcoder {
	stream:  usize,
	filter:  filter::Graph,
	decoder: codec::decoder::Audio,
	encoder: codec::encoder::Audio,
}

fn transcoder<P: AsRef<Path>>(ictx: &mut format::context::Input, octx: &mut format::context::Output, path: &P, filter_spec: &str) -> Result<Transcoder, ffmpeg::Error> {
	let input   = ictx.streams().best(media::Type::Audio).expect("could not find best audio stream");
	let decoder = try!(input.codec().decoder().audio());
	let codec   = try!(ffmpeg::encoder::find(octx.format().codec(path, media::Type::Audio)).expect("failed to find encoder").audio());
	let global  = octx.format().flags().contains(ffmpeg::format::flag::GLOBAL_HEADER);

	let mut output  = try!(octx.add_stream(codec));
	let mut encoder = try!(output.codec().encoder().audio());

	let channel_layout = codec.channel_layouts()
		.map(|cls| cls.best(decoder.channel_layout().channels()))
		.unwrap_or(ffmpeg::channel_layout::STEREO);

	if global {
		encoder.set_flags(ffmpeg::codec::flag::GLOBAL_HEADER);
	}

	encoder.set_rate(decoder.rate() as i32);
	encoder.set_channel_layout(channel_layout);
	encoder.set_channels(channel_layout.channels());
	encoder.set_format(codec.formats().expect("unknown supported formats").next().unwrap());
	encoder.set_bit_rate(decoder.bit_rate());
	encoder.set_max_bit_rate(decoder.max_bit_rate());

	encoder.set_time_base((1, decoder.rate() as i32));
	output.set_time_base((1, decoder.rate() as i32));

	let encoder = try!(encoder.open_as(codec));
	let filter  = try!(filter(filter_spec, &decoder, &encoder));

	Ok(Transcoder {
		stream:  input.index(),
		filter:  filter,
		decoder: decoder,
		encoder: encoder,
	})
}

// Transcode the `best` audio stream of the input file into a the output file while applying a
// given filter. If no filter was specified the stream gets copied (`anull` filter).
//
// Example 1: Transcode *.mp3 file to *.wmv while speeding it up
// transcode-audio in.mp3 out.wmv "atempo=1.2"
//
// Example 2: Overlay an audio file
// transcode-audio in.mp3 out.mp3 "amovie=overlay.mp3 [ov]; [in][ov] amerge [out]"
//
// Example 3: Seek to a specified position (in seconds)
// transcode-audio in.mp3 out.mp3 anull 30
fn main() {
	ffmpeg::init().unwrap();

	let input  = env::args().nth(1).expect("missing input");
	let output = env::args().nth(2).expect("missing output");
	let filter = env::args().nth(3).unwrap_or("anull".to_owned());
	let seek   = env::args().nth(4).and_then(|s| s.parse::<i64>().ok());

	let mut ictx       = format::input(&input).unwrap();
	let mut octx       = format::output(&output).unwrap();
	let mut transcoder = transcoder(&mut ictx, &mut octx, &output, &filter).unwrap();

	if let Some(position) = seek {
		// If the position was given in seconds, rescale it to ffmpegs base timebase.
		let position = position.rescale((1, 1), rescale::TIME_BASE);
		// If this seek was embedded in the transcoding loop, a call of `flush()`
		// for every opened buffer after the successful seek would be advisable.
		ictx.seek(position, ..position).unwrap();
	}

	octx.set_metadata(ictx.metadata().to_owned());
	octx.write_header().unwrap();

	let in_time_base  = transcoder.decoder.time_base();
	let out_time_base = octx.stream(0).unwrap().time_base();

	let mut decoded = frame::Audio::empty();
	let mut encoded = ffmpeg::Packet::empty();

	for (stream, mut packet) in ictx.packets() {
		if stream.index() == transcoder.stream {
			packet.rescale_ts(stream.time_base(), in_time_base);

			if let Ok(true) = transcoder.decoder.decode(&packet, &mut decoded) {
				let timestamp = decoded.timestamp();
				decoded.set_pts(timestamp);

				transcoder.filter.get("in").unwrap().source().add(&decoded).unwrap();

				while let Ok(..) = transcoder.filter.get("out").unwrap().sink().frame(&mut decoded) {
					if let Ok(true) = transcoder.encoder.encode(&decoded, &mut encoded) {
						encoded.set_stream(0);
						encoded.rescale_ts(in_time_base, out_time_base);
						encoded.write_interleaved(&mut octx).unwrap();
					}
				}
			}
		}
	}

	transcoder.filter.get("in").unwrap().source().flush().unwrap();

	while let Ok(..) = transcoder.filter.get("out").unwrap().sink().frame(&mut decoded) {
		if let Ok(true) = transcoder.encoder.encode(&decoded, &mut encoded) {
			encoded.set_stream(0);
			encoded.rescale_ts(in_time_base, out_time_base);
			encoded.write_interleaved(&mut octx).unwrap();
		}
	}

	if let Ok(true) = transcoder.encoder.flush(&mut encoded) {
		encoded.set_stream(0);
		encoded.rescale_ts(in_time_base, out_time_base);
		encoded.write_interleaved(&mut octx).unwrap();
	}

	octx.write_trailer().unwrap();
}
