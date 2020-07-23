extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();

    if let Ok(mut ictx) = input(&env::args().nth(1).expect("Cannot open file.")) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or_else(|| ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let mut decoder = input.codec().decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let mut frame_index = 0;
        for (stream, packet) in ictx.packets() {
            if stream.index() != video_stream_index {
                continue;
            }
            let mut frame = Video::empty();
            match decoder.decode(&packet, &mut frame) {
                Ok(_) => {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&frame, &mut rgb_frame)?;
                    match save_file(&rgb_frame, frame_index) {
                        Ok(_) => {}
                        Err(e) => println!("Error occurred during file writing - {}", e),
                    }
                    frame_index += 1;
                }
                _ => {
                    println!("Error occurred while decoding packet.");
                }
            }
        }
    }

    Ok(())
}

fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    let mut file = File::create(format!("frame{}.ppm", index))?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}
