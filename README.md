[![crates.io](https://img.shields.io/crates/v/ffmpeg-next.svg)](https://crates.io/crates/ffmpeg-next)
[![docs.rs](https://docs.rs/ffmpeg-next/badge.svg)](https://docs.rs/ffmpeg-next/)
[![build](https://github.com/zmwangx/rust-ffmpeg/workflows/build/badge.svg)](https://github.com/zmwangx/rust-ffmpeg/actions)

This is a fork of the abandoned [ffmpeg](https://crates.io/crates/ffmpeg) crate by [meh.](https://github.com/meh/rust-ffmpeg).

Currently supported FFmpeg versions: 3.4.x through 4.4.x.

Build instructions can be found on the [wiki](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building).

Documentation:

- [docs.rs](https://docs.rs/ffmpeg-next/);
- [FFmpeg user manual](https://ffmpeg.org/ffmpeg-all.html);
- [FFmpeg Doxygen](https://ffmpeg.org/doxygen/trunk/).

*Note on upgrading to v4.3.4 or later: v4.3.4 introduced automatic FFmpeg version detection, obsoleting feature flags `ffmpeg4`, `ffmpeg41`, `ffmpeg42` and `ffmpeg43`. If you manually specify any of these features, now is the time to remove them; if you use `ffmpeg43` through the `default` feature, it's still on for backward-compatibility but it has turned into a no-op, and you don't need to do anything. Deprecation plan: `ffmpeg43` will be dropped from default features come 4.4, and all these features will be removed come 5.0.*

*See [CHANGELOG.md](CHANGELOG.md) for other information on version upgrades.*

A word on versioning: major and minor versions of this crate track major and minor versions of FFmpeg, e.g. 4.2.x of this crate has been updated to support the 4.2.x series of FFmpeg. Patch level is reserved for changes to this crate and does not track FFmpeg patch versions. Since we can only freely bump the patch level, versioning of this crate differs from semver: minor versions may behave like semver major versions and introduce backward-incompatible changes; patch versions may behave like semver minor versions and introduce new APIs. Please peg the version you use accordingly.

**Please realize that this crate is in maintenance-only mode for the most part.** Which means I'll try my best to ensure the crate compiles against all release branches of FFmpeg 3.4 and later (only the latest patch release of each release branch is officially supported) and fix reported bugs, but if a new FFmpeg version brings new APIs that require significant effort to port to Rust, you might have to send me a PR (and just to be clear, I can't really guarantee I'll have the time to review). Any PR to improve existing API is unlikely to be merged, unfortunately.

🤝 **If you have significant, demonstrable experience in Rust and multimedia-related programming, please let me know, I'll be more than happy to invite you as a collaborator.** 🤝
