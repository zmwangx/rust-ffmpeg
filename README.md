[![crates.io](https://img.shields.io/crates/v/ffmpeg-next.svg)](https://crates.io/crates/ffmpeg-next)
[![docs.rs](https://docs.rs/ffmpeg-next/badge.svg)](https://docs.rs/ffmpeg-next/)
[![build](https://github.com/zmwangx/rust-ffmpeg/workflows/build/badge.svg)](https://github.com/zmwangx/rust-ffmpeg/actions)

This is a fork of the [ffmpeg](https://crates.io/crates/ffmpeg) crate hosted at [meh/rust-ffmpeg](https://github.com/meh/rust-ffmpeg).
ffmpeg-next was created due to the stalled development of the ffmpeg crate.

**This crate is in maintenance-only mode.**

This means:
 - the crate will continue to compile for all release branches of FFmpeg 3.4 and later (only the latest patch release of each release branch is officially supported)
 - bug fixes will be merged
 - new APIs wil **not** be merged
 - efforts to improve existig API are **unlikely** to be merged.

This crate is currently in maintenance mode, and aims to be compatible with all of FFmpeg's versions from 3.4
(currently from 3.4 til 8.0)

Build instructions can be found on the [wiki](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building).

Documentation:

- [docs.rs](https://docs.rs/ffmpeg-next/);
- [FFmpeg user manual](https://ffmpeg.org/ffmpeg-all.html);
- [FFmpeg Doxygen](https://ffmpeg.org/doxygen/trunk/).

*Note on upgrading to v4.3.4 or later: v4.3.4 introduced automatic FFmpeg version detection, obsoleting feature flags `ffmpeg4`, `ffmpeg41`, `ffmpeg42` and `ffmpeg43`. If you manually specify any of these features, now is the time to remove them; if you use `ffmpeg43` through the `default` feature, it's still on for backward-compatibility but it has turned into a no-op, and you don't need to do anything. Deprecation plan: `ffmpeg43` will be dropped from default features come 4.4, and all these features will be removed come 5.0.*

*See [CHANGELOG.md](CHANGELOG.md) for other information on version upgrades.*

A word on versioning: major and minor versions of this crate track major and minor versions of FFmpeg, e.g. 4.2.x of this crate has been updated to support the 4.2.x series of FFmpeg. Patch level is reserved for changes to this crate and does not track FFmpeg patch versions. Since we can only freely bump the patch level, versioning of this crate differs from semver: minor versions may behave like semver major versions and introduce backward-incompatible changes; patch versions may behave like semver minor versions and introduce new APIs. Please peg the version you use accordingly.

🤝 **If you have significant, demonstrable experience in Rust and multimedia-related programming, please let me know, I'll be more than happy to invite you as a collaborator.** 🤝
