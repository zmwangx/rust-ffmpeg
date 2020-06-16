[![crates.io](https://img.shields.io/crates/v/ffmpeg-next.svg)](https://crates.io/crates/ffmpeg-next)

This is a fork of the abandoned [ffmpeg](https://crates.io/crates/ffmpeg) crate by [meh.](https://github.com/meh/rust-ffmpeg).

Support for different FFmpeg versions are guarded by feature flags:

| FFmpeg version | lavc version | corresponding feature        |
| -------------- | ------------ | ---------------------------- |
| 4.3.x          | 58.91.100    | `ffmpeg43` (current default) |
| 4.2.x          | 58.54.100    | `ffmpeg42`                   |
| 4.1.x          | 58.35.100    | `ffmpeg41`                   |
| 4.0.x          | 58.18.100    | `ffmpeg4`                    |
| 3.4.x          | 57.107.100   | none                         |

See my [`metadata` project](https://github.com/zmwangx/metadata) for an example of targeting multiple versions of FFmpeg.

A word on versioning: major and minor versions of this crate track major and minor versions of FFmpeg, e.g. 4.2.x of this crate has been updated to support the 4.2.x series of FFmpeg. Patch level is reserved for bug fixes of this crate and does not track FFmpeg patch versions.
