extern crate ffmpeg;

use std::env;
use std::path::Path;

use ffmpeg::{format, codec, frame, media, filter};
use ffmpeg::option::Settable;

fn filter(spec: &str, decoder: &codec::decoder::Audio, encoder: &codec::encoder::audio::Encoder) -> Result<filter::Graph, ffmpeg::Error> {
	let mut filter = filter::Graph::new();

	let base = ffmpeg::Rational(1, 1000000);
	let args = format!("time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
		base, decoder.rate(), decoder.format().name(), decoder.channel_layout().bits());

	try!(filter.add(&filter::find("abuffer").unwrap(), "in", &args));
	try!(filter.add(&filter::find("abuffersink").unwrap(), "out", ""));

	{
		let mut sink = filter.get("out").unwrap();

		sink.set_sample_format(encoder.format()).unwrap();
		sink.set_channel_layout(encoder.channel_layout()).unwrap();
		sink.set_sample_rate(encoder.rate()).unwrap();
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
	encoder: codec::encoder::audio::Encoder,
}

fn transcoder<P: AsRef<Path>>(ictx: &mut format::context::Input, octx: &mut format::context::Output, path: &P, filter_spec: &str) -> Result<Transcoder, ffmpeg::Error> {
	let input   = ictx.streams().best(media::Type::Audio).expect("could not find best audio stream");
	let decoder = try!(input.codec().decoder().audio());
	let codec   = try!(ffmpeg::encoder::find(octx.format().codec(path, media::Type::Audio)).expect("failed to find encoder").audio());

	let mut output  = octx.add_stream(&codec);
	let mut encoder = try!(output.codec().encoder().audio());

	encoder.set_rate(decoder.rate() as i32);
	encoder.set_channel_layout(decoder.channel_layout());
	encoder.set_channels(decoder.channel_layout().channels());
	encoder.set_format(codec.formats().expect("unknown supported formats").next().unwrap());

	output.set_time_base((1, decoder.rate() as i32));

	let encoder = try!(encoder.open_as(&codec));
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
fn main() {
	ffmpeg::init().unwrap();

	let input  = env::args().nth(1).expect("missing input");
	let output = env::args().nth(2).expect("missing output");
	let filter = env::args().nth(3).unwrap_or("anull".to_owned());

	let mut ictx       = format::input(&input).unwrap();
	let mut octx       = format::output(&output).unwrap();
	let mut transcoder = transcoder(&mut ictx, &mut octx, &output, &filter).unwrap();

	octx.set_metadata(ictx.metadata().to_owned());
	octx.write_header().unwrap();

	let     time_base = (1, 1000000);
	let mut frame     = frame::Audio::empty();

	for (stream, mut packet) in ictx.packets() {
		if stream.index() == transcoder.stream {
			let (os_index, os_time_base) = {
				let os = octx.stream(stream.index()).unwrap();
				(os.index(), os.time_base())
			};

			packet.rescale_ts(stream.time_base(), time_base);

			if transcoder.decoder.decode(&packet, &mut frame).unwrap() {
				let timestamp = frame.timestamp();
				frame.set_pts(timestamp);
				transcoder.filter.get("in").unwrap().source().add(&frame).unwrap();

				while let Ok(..) = transcoder.filter.get("out").unwrap().sink().frame(&mut frame) {
					let mut encoded = ffmpeg::Packet::empty();

					if let Ok(true) = transcoder.encoder.encode(&frame, &mut encoded) {
						encoded.set_stream(os_index);
						encoded.rescale_ts(time_base, os_time_base);
						encoded.write_interleaved(&mut octx).unwrap();
					}
				}
			}
		}
	}

	octx.write_trailer().unwrap();
}
