[![crates.io](https://img.shields.io/crates/v/ffmpeg-next.svg)](https://crates.io/crates/ffmpeg-next)
[![build](https://github.com/zmwangx/rust-ffmpeg/workflows/build/badge.svg)](https://github.com/zmwangx/rust-ffmpeg/actions)

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

If you have problem building this crate, please have a look at the [wiki](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building).

**Please realize that this crate is in maintenance-only mode for the most part.** Which means I'll try my best to ensure the crate compiles against all release branches of FFmpeg 3.4 and later (only the latest patch release of each release branch is officially supported) and fix reported bugs, but if a new FFmpeg version brings new APIs that require significant effort to port to Rust, you might have to send me a PR (and just to be clear, I can't really guarantee I'll have the time to review). Any PR to improve existing API is unlikely to be merged, unfortunately.

🤝 **If you have significant, demonstrable experience in Rust and multimedia-related programming, please let me know, I'll be more than happy to invite you as a collaborator.** 🤝
