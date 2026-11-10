extern crate ffmpeg_next as ffmpeg;

use crate::ffmpeg::{filter, format, frame};

fn build_graph() -> filter::Graph {
    let mut graph = filter::Graph::new();
    graph
        .add(
            &filter::find("buffer").unwrap(),
            "in",
            "video_size=16x16:pix_fmt=yuv420p:time_base=1/25:pixel_aspect=1/1",
        )
        .unwrap();
    graph
        .add(&filter::find("buffersink").unwrap(), "out", "")
        .unwrap();
    graph
        .output("in", 0)
        .unwrap()
        .input("out", 0)
        .unwrap()
        .parse("null")
        .unwrap();
    graph.validate().unwrap();
    graph
}

// `Source::add` takes `&Frame`, so the caller's frame must keep its buffers.
#[test]
fn buffersrc_add_leaves_caller_frame_intact() {
    ffmpeg::init().unwrap();

    let mut graph = build_graph();
    let mut input = frame::Video::new(format::Pixel::YUV420P, 16, 16);
    input.data_mut(0).fill(0xAB);
    input.set_pts(Some(0));

    graph.get("in").unwrap().source().add(&input).unwrap();

    assert_eq!(input.width(), 16);
    assert_eq!(input.format(), format::Pixel::YUV420P);
    assert_eq!(input.planes(), 3);

    let mut output = frame::Video::empty();
    graph.get("out").unwrap().sink().frame(&mut output).unwrap();
    assert_eq!(output.data(0).as_ptr(), input.data(0).as_ptr());
    drop(output);

    assert!(input.data(0).iter().all(|&byte| byte == 0xAB));
}
