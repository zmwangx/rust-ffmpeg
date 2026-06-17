extern crate ffmpeg_next as ffmpeg;

use ffmpeg::{ffi, filter, format, frame};

fn build_graph(width: u32, height: u32, pix_fmt: format::Pixel) -> filter::Graph {
    let mut graph = filter::Graph::new();

    let args = format!(
        "video_size={}x{}:pix_fmt={}:time_base=1/25:pixel_aspect=1/1",
        width,
        height,
        ffi::AVPixelFormat::from(pix_fmt) as i32,
    );

    graph
        .add(&filter::find("buffer").unwrap(), "in", &args)
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

fn push_frames(graph: &mut filter::Graph, count: i64, width: u32, height: u32, fmt: format::Pixel) {
    for pts in 0..count {
        let mut input = frame::Video::new(fmt, width, height);
        input.set_pts(Some(pts));
        graph.get("in").unwrap().source().add(&input).unwrap();
    }
    graph.get("in").unwrap().source().flush().unwrap();
}

// A single reused `Frame` drains every queued frame.
#[test]
fn buffersink_frame_reuse_drains_all_frames() {
    ffmpeg::init().unwrap();

    const W: u32 = 16;
    const H: u32 = 16;
    const COUNT: i64 = 8;
    let fmt = format::Pixel::RGB24;

    let mut graph = build_graph(W, H, fmt);
    push_frames(&mut graph, COUNT, W, H, fmt);

    let mut reused = frame::Video::empty();
    let mut drained = 0;
    while graph.get("out").unwrap().sink().frame(&mut reused).is_ok() {
        assert_eq!(reused.width(), W);
        assert_eq!(reused.height(), H);
        drained += 1;
    }

    assert_eq!(drained, COUNT);
}

// Pulling a second frame into the same `Frame` must release the first buffer:
// pin an extra ref to it and check its ref count drops by one on reuse.
#[test]
fn buffersink_frame_reuse_releases_previous_buffer() {
    ffmpeg::init().unwrap();

    const W: u32 = 16;
    const H: u32 = 16;
    let fmt = format::Pixel::RGB24;

    let mut graph = build_graph(W, H, fmt);
    push_frames(&mut graph, 2, W, H, fmt);

    let mut reused = frame::Video::empty();

    graph.get("out").unwrap().sink().frame(&mut reused).unwrap();

    unsafe {
        let first_buf = (*reused.as_ptr()).buf[0];
        assert!(!first_buf.is_null(), "drained frame must be refcounted");

        let mut pinned = ffi::av_buffer_ref(first_buf);
        let before = ffi::av_buffer_get_ref_count(pinned);

        graph.get("out").unwrap().sink().frame(&mut reused).unwrap();
        let after = ffi::av_buffer_get_ref_count(pinned);

        ffi::av_buffer_unref(&mut pinned);

        assert_eq!(
            after,
            before - 1,
            "reused frame did not release its previous buffer (leak)"
        );
    }
}
