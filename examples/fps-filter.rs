extern crate ffmpeg_next as ffmpeg;

use ffmpeg::*;
use std::env;

const DEFAULT_INPUT: &str = "input.gif";
const DEFAULT_OUTPUT: &str = "output.gif";
const DEFAULT_TARGET_FPS: f64 = 25.0;

fn main() {
    let input_file = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_INPUT.to_string());
    let output_file = env::args()
        .nth(2)
        .unwrap_or_else(|| DEFAULT_OUTPUT.to_string());
    let target_fps: f64 = env::args()
        .nth(3)
        .unwrap_or_else(|| DEFAULT_TARGET_FPS.to_string())
        .parse()
        .unwrap();

    init()
        .map_err(|e| format!("Unable to initialize ffmpeg: {}", e))
        .unwrap();
    util::log::set_level(util::log::Level::Trace);
    let mut input_context = format::input(&input_file)
        .map_err(|e| format!("Unable to open input file: {}", e))
        .unwrap();
    let mut output_context = format::output(&output_file)
        .map_err(|e| format!("Unable to open output file: {}", e))
        .unwrap();

    let stream = input_context
        .streams()
        .best(media::Type::Video)
        .ok_or("The file has no video tracks")
        .unwrap();
    let stream_index = stream.index();
    let mut decoder = stream
        .codec()
        .decoder()
        .video()
        .map_err(|e| format!("Unable to decode the codec used in the video: {}", e))
        .unwrap();

    let format_aspect_ratio = |sar: util::rational::Rational| match sar.numerator() {
        0 => "1".to_string(),
        _ => format!("{}/{}", sar.numerator(), sar.denominator()),
    };
    let buffer_args = format!(
        "width={}:height={}:pix_fmt={}:time_base={}:sar={}",
        decoder.width(),
        decoder.height(),
        decoder.format().descriptor().unwrap().name(),
        stream.time_base(),
        format_aspect_ratio(decoder.aspect_ratio()),
    );
    let mut filter = filter::Graph::new();
    filter
        .add(&filter::find("buffer").unwrap(), "in", &buffer_args)
        .unwrap();
    filter
        .add(&filter::find("buffersink").unwrap(), "out", "")
        .unwrap();
    filter
        .output("in", 0)
        .unwrap()
        .input("out", 0)
        .unwrap()
        .parse(&format!("fps=fps={},format=bgr8", target_fps)[..])
        .unwrap();
    filter.validate().unwrap();

    let codec = ffmpeg::encoder::find(
        output_context
            .format()
            .codec(&DEFAULT_OUTPUT, media::Type::Video),
    )
    .expect("Unable to find encoder")
    .video()
    .unwrap();
    let mut output_stream = output_context.add_stream(codec).unwrap();
    let mut encoder = output_stream.codec().encoder().video().unwrap();
    encoder.set_format(format::Pixel::BGR8);
    encoder.set_width(decoder.width());
    encoder.set_height(decoder.height());
    encoder.set_time_base((1, 100));
    output_stream.set_time_base((1, 100));
    let mut encoder = encoder.open_as(codec).unwrap();
    output_stream.set_parameters(&encoder);
    output_context.write_header().unwrap();

    let mut input_frame_count = 0;
    let mut output_frame_count = 0;
    let mut input_pts = 0;
    let mut output_pts = 0;

    let mut write_frame = |rgba_encoded: &mut Packet| {
        rgba_encoded.set_stream(0);
        rgba_encoded.set_pts(Option::from(output_pts));
        rgba_encoded.write_interleaved(&mut output_context).unwrap();
        output_pts += (1.0 / target_fps * 100.0) as i64;
        output_frame_count += 1;
    };

    let mut process_encoded_packets = |encoder: &mut encoder::Video| {
        let mut rgba_encoded = Packet::empty();
        while encoder.receive_packet(&mut rgba_encoded).is_ok() {
            write_frame(&mut rgba_encoded);
        }
    };

    let mut process_decoded_frames =
        |decoder: &mut decoder::Video, encoder: &mut encoder::Video| {
            let mut input_frame = frame::Video::empty();
            while match decoder.receive_frame(&mut input_frame) {
                Ok(_) => true,
                Err(e) => {
                    if e != (Error::Other {
                        errno: error::EAGAIN,
                    }) {
                        eprintln!("receive_frame error: {}", e);
                    }
                    false
                }
            } {
                input_frame_count += 1;
                let mut rgba_frame = frame::Video::empty();
                filter
                    .get("in")
                    .unwrap()
                    .source()
                    .add(&input_frame)
                    .unwrap();
                while filter
                    .get("out")
                    .unwrap()
                    .sink()
                    .frame(&mut rgba_frame)
                    .is_ok()
                {
                    encoder.send_frame(&rgba_frame).unwrap();
                    process_encoded_packets(encoder);
                }
            }
        };

    for (s, packet) in input_context.packets() {
        if s.index() != stream_index {
            continue;
        }
        input_pts += packet.duration();
        decoder.send_packet(&packet).unwrap();
        process_decoded_frames(&mut decoder, &mut encoder);
        // if !decoder.decode(&packet, &mut input_frame).unwrap() {
        //     continue;
        // }
        // input_frame_count += 1;

        // let mut rgba_frame = util::frame::video::Video::empty();
        // let mut rgba_encoded = Packet::empty();
        // filter
        //     .get("in")
        //     .unwrap()
        //     .source()
        //     .add(&input_frame)
        //     .unwrap();
        // while filter
        //     .get("out")
        //     .unwrap()
        //     .sink()
        //     .frame(&mut rgba_frame)
        //     .is_ok()
        // {
        //     if let Ok(true) = encoder.encode(&rgba_frame, &mut rgba_encoded) {
        //         write_frame(&mut rgba_encoded);
        //     }
        // }
    }
    decoder.send_eof().unwrap();
    process_decoded_frames(&mut decoder, &mut encoder);

    filter.get("in").unwrap().source().close(input_pts).unwrap();
    let mut rgba_frame = frame::Video::empty();
    while filter
        .get("out")
        .unwrap()
        .sink()
        .frame(&mut rgba_frame)
        .is_ok()
    {
        encoder.send_frame(&rgba_frame).unwrap();
        process_encoded_packets(&mut encoder);
    }

    encoder.send_eof().unwrap();
    process_encoded_packets(&mut encoder);

    // // let mut rgba_frame = util::frame::video::Video::empty();
    // // let mut rgba_encoded = Packet::empty();
    // filter.get("in").unwrap().source().close(input_pts).unwrap();
    // while filter
    //     .get("out")
    //     .unwrap()
    //     .sink()
    //     .frame(&mut rgba_frame)
    //     .is_ok()
    // {
    //     if let Ok(true) = encoder.encode(&rgba_frame, &mut rgba_encoded) {
    //         write_frame(&mut rgba_encoded);
    //     }
    // }
    // if let Ok(true) = encoder.flush(&mut rgba_encoded) {
    //     write_frame(&mut rgba_encoded);
    // }
    output_context.write_trailer().unwrap();

    println!("Total input frames: {}", input_frame_count);
    println!("Total output frames: {}", output_frame_count);
    println!("Total duration: {:.2}s", output_pts as f64 / 100.0);
}
