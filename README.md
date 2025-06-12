# rust-ffmpeg

[![crates.io](https://img.shields.io/crates/v/rust-ffmpeg)](https://crates.io/crates/rust-ffmpeg)
[![docs.rs](https://docs.rs/rust-ffmpeg/badge.svg)](https://docs.rs/rust-ffmpeg)
[![build](https://github.com/nrbnlulu/rust-ffmpeg/workflows/build/badge.svg)](https://github.com/nrbnlulu/rust-ffmpeg/actions)

currently supports ffmpeg 6.1 - 7.1

Build instructions can be found on the [wiki](https://github.com/nrbnlulu/rust-ffmpeg/wiki/Notes-on-building).

Documentation:

- [docs.rs](https://docs.rs/rust-ffmpeg/);
- [FFmpeg user manual](https://ffmpeg.org/ffmpeg-all.html);
- [FFmpeg Doxygen](https://ffmpeg.org/doxygen/trunk/).

### Installation

As a general note take a look on the build.yml
for a better reference on how to build the library.

#### Windows

1. download the version you want from [ffmpeg-builds](https://github.com/BtbN/FFmpeg-Builds/releases) (only shared builds are supported)
2. extract and set `FFMPEG_DIR` `FFMPEG_INCLUDE_DIR` and `FFMPEG_LIB_DIR` environment variables to the extracted directory

#### Linux

1. download the version you want from [ffmpeg-builds](https://github.com/BtbN/FFmpeg-Builds/releases) (only shared builds are supported)
2. extract and set `FFMPEG_DIR` `FFMPEG_INCLUDE_DIR` and `FFMPEG_LIB_DIR` environment variables to the extracted directory

#### MacOS

1. run `brew install ffmpeg pkg-config`
2. good to go
