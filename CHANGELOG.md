4.3.6
-----

- util: fix Windows compatibility due to unavailable errnos. (#30)

4.3.5
-----

- util: add `util::log` module to expose FFmpeg's logging facilities.

- filter: add method `Source::close()` to expose `av_buffersrc_close`. (#23)

- codec: add new encoding/decoding APIs `send_frame()` / `send_eof()`, `receive_packet()` to `encoder::{Audio, Video}` and `send_packet()` / `send_eof()`, `receive_frame()` to `decoder::{Audio, Video}` based on modern send/receive APIs (instead of `avcodec_decode_video2()` / `avcodec_decode_audio4()` / `avcodec_encode_video2()` /`avcodec_encode_audio2()` which have been deprecated since FFmpeg 3.1). Users should consider switching to the new APIs. See [documentation in `libavcodec/avcodec.h`](https://github.com/FFmpeg/FFmpeg/blob/n4.3.1/libavcodec/avcodec.h#L84-L196) for details. (#28)

- util: introduce new `Error` variant `Error::Other { errno }` for wrapped POSIX error codes (see the `AVERROR` macro in `libavutil/error.h`), and reexport common POSIX error codes under `util::error`. (#24)

4.3.4
-----

- crate: FFmpeg version detection is now automatic, obseleting feature flags `ffmpeg4`, `ffmpeg41`, `ffmpeg42` and `ffmpeg43`. The flags are kept as noop for now, will be removed in 5.0.
