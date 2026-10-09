// Transcodes the best video stream to H.264 using Intel Quick Sync Video
// while copying audio and subtitle streams.
//
// Invocation:
//
//   cargo run --example transcode-qsv -- input.mp4 output.mp4 [device]
//
// FFmpeg must provide the h264_qsv encoder and a usable QSV device. If no
// backend-specific device name is supplied, FFmpeg selects its default.

extern crate ffmpeg_next as ffmpeg;

use std::env;

use ffmpeg::software::scaling::{context::Context as Scaler, flag::Flags as ScalerFlags};
use ffmpeg::{
    Dictionary, Packet, Rational, codec, decoder, encoder, format, frame, hardware, media,
};

const INITIAL_POOL_SIZE: usize = 32;
const ICQ_QUALITY: i32 = 23;

struct Transcoder {
    output_stream: usize,
    input_time_base: Rational,
    decoder: decoder::Video,
    scaler: Scaler,
    frames: hardware::Frames,
    encoder: encoder::Video,
}

impl Transcoder {
    fn new(
        input: &format::stream::Stream,
        output: &mut format::context::Output,
        output_stream: usize,
        device_name: Option<&str>,
    ) -> Result<Self, ffmpeg::Error> {
        let global_header = output
            .format()
            .flags()
            .contains(format::Flags::GLOBAL_HEADER);
        let input_time_base = input.time_base();
        let decoder = codec::context::Context::from_parameters(input.parameters())?
            .decoder()
            .video()?;
        let codec = encoder::find_by_name("h264_qsv").ok_or(ffmpeg::Error::EncoderNotFound)?;
        let device = hardware::Device::create(hardware::Type::Qsv, device_name)?;
        let frames = device.frames_with_pool_size(
            format::Pixel::QSV,
            format::Pixel::NV12,
            decoder.width(),
            decoder.height(),
            INITIAL_POOL_SIZE,
        )?;
        let scaler = Scaler::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            format::Pixel::NV12,
            decoder.width(),
            decoder.height(),
            ScalerFlags::BILINEAR,
        )?;

        let mut encoder = codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()?;
        encoder.set_width(decoder.width());
        encoder.set_height(decoder.height());
        encoder.set_aspect_ratio(decoder.aspect_ratio());
        encoder.set_time_base(input_time_base);
        encoder.set_gop(50);
        encoder.set_max_b_frames(0);
        encoder.set_global_quality(ICQ_QUALITY);
        encoder.set_hardware_device(&device)?;
        encoder.set_hardware_frames(&frames)?;

        let frame_rate = decoder
            .frame_rate()
            .or_else(|| {
                let rate = input.avg_frame_rate();
                if rate.numerator() > 0 && rate.denominator() > 0 {
                    Some(rate)
                } else {
                    None
                }
            })
            .or_else(|| {
                let rate = input.rate();
                if rate.numerator() > 0 && rate.denominator() > 0 {
                    Some(rate)
                } else {
                    None
                }
            });
        encoder.set_frame_rate(frame_rate);

        if global_header {
            encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        }

        let mut options = Dictionary::new();
        options.set("preset", "slower");
        let encoder = encoder.open_as_with(codec, options)?;
        let mut stream = output.add_stream(codec)?;
        stream.set_parameters(&encoder);
        stream.set_time_base(input_time_base);
        if let Some(frame_rate) = frame_rate {
            stream.set_rate(frame_rate);
            stream.set_avg_frame_rate(frame_rate);
        }

        Ok(Transcoder {
            output_stream,
            input_time_base,
            decoder,
            scaler,
            frames,
            encoder,
        })
    }

    fn send_packet(&mut self, packet: &Packet) -> Result<(), ffmpeg::Error> {
        self.decoder.send_packet(packet)
    }

    fn send_decoder_eof(&mut self) -> Result<(), ffmpeg::Error> {
        self.decoder.send_eof()
    }

    fn send_encoder_eof(&mut self) -> Result<(), ffmpeg::Error> {
        self.encoder.send_eof()
    }

    fn process_frames(
        &mut self,
        output: &mut format::context::Output,
        output_time_base: Rational,
    ) -> Result<(), ffmpeg::Error> {
        let mut decoded = frame::Video::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut software = frame::Video::empty();
            self.scaler.run(&decoded, &mut software)?;
            software.set_pts(decoded.timestamp().or_else(|| decoded.pts()));

            let hardware = self.frames.upload(&software)?;
            self.encoder.send_frame(&hardware)?;
            self.write_packets(output, output_time_base)?;
        }

        Ok(())
    }

    fn write_packets(
        &mut self,
        output: &mut format::context::Output,
        output_time_base: Rational,
    ) -> Result<(), ffmpeg::Error> {
        let mut packet = Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.output_stream);
            packet.rescale_ts(self.input_time_base, output_time_base);
            packet.write_interleaved(output)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), ffmpeg::Error> {
    let input_path = env::args().nth(1).expect("missing input file");
    let output_path = env::args().nth(2).expect("missing output file");
    let device_name = env::args().nth(3);

    ffmpeg::init()?;

    let mut input = format::input(&input_path)?;
    let mut output = format::output(&output_path)?;
    let video_stream = input
        .streams()
        .best(media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?
        .index();

    let mut stream_mapping = vec![-1; input.nb_streams() as usize];
    let mut input_time_bases = vec![Rational(0, 1); input.nb_streams() as usize];
    let mut transcoder = None;
    let mut output_stream = 0;

    for (input_stream, stream) in input.streams().enumerate() {
        let medium = stream.parameters().medium();
        let should_copy = medium == media::Type::Audio || medium == media::Type::Subtitle;

        if input_stream == video_stream {
            stream_mapping[input_stream] = output_stream as isize;
            input_time_bases[input_stream] = stream.time_base();
            transcoder = Some(Transcoder::new(
                &stream,
                &mut output,
                output_stream,
                device_name.as_deref(),
            )?);
            output_stream += 1;
        } else if should_copy {
            stream_mapping[input_stream] = output_stream as isize;
            input_time_bases[input_stream] = stream.time_base();

            let mut copied = output.add_stream(encoder::find(codec::Id::None))?;
            copied.set_parameters(stream.parameters());
            unsafe {
                (*copied.parameters().as_mut_ptr()).codec_tag = 0;
            }
            output_stream += 1;
        }
    }

    output.set_metadata(input.metadata().to_owned());
    output.write_header()?;

    let output_time_bases: Vec<_> = output.streams().map(|stream| stream.time_base()).collect();
    let transcoder = transcoder.as_mut().ok_or(ffmpeg::Error::StreamNotFound)?;

    for (stream, mut packet) in input.packets() {
        let input_stream = stream.index();
        let mapped_stream = stream_mapping[input_stream];
        if mapped_stream < 0 {
            continue;
        }

        let mapped_stream = mapped_stream as usize;
        if input_stream == video_stream {
            transcoder.send_packet(&packet)?;
            transcoder.process_frames(&mut output, output_time_bases[mapped_stream])?;
        } else {
            packet.rescale_ts(
                input_time_bases[input_stream],
                output_time_bases[mapped_stream],
            );
            packet.set_position(-1);
            packet.set_stream(mapped_stream);
            packet.write_interleaved(&mut output)?;
        }
    }

    transcoder.send_decoder_eof()?;
    transcoder.process_frames(&mut output, output_time_bases[transcoder.output_stream])?;
    transcoder.send_encoder_eof()?;
    transcoder.write_packets(&mut output, output_time_bases[transcoder.output_stream])?;
    output.write_trailer()?;

    Ok(())
}
