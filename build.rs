use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

fn output() -> PathBuf {
	PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn feature(header: &str, feature: Option<&str>, var: &str) -> io::Result<()> {
	if let Some(feature) = feature {
		if env::var(format!("CARGO_FEATURE_{}", feature.to_uppercase())).is_err() {
			return Ok(());
		}
	}

	try!(write!(try!(File::create(output().join("check.c"))), r#"
		#include <stdio.h>
		#include <{}>

		int
		main (int argc, char* argv[])
		{{
			printf("%d\n", {});
			return 0;
		}}
	"#, header, var));

	if Command::new("cc").current_dir(&output()).arg("-o").arg("check").arg("check.c").status().is_err() {
		return Ok(());
	}

	if try!(Command::new("./check").current_dir(&output()).output()).stdout[0] == b'1' {
		println!(r#"cargo:rustc-cfg=feature="{}""#, var.to_lowercase());
	}

	Ok(())
}

fn main() {
	if env::var("CARGO_FEATURE_BUILD").is_ok() {
		return;
	}

	feature("libavutil/avutil.h", None, "FF_API_OLD_AVOPTIONS").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_PIX_FMT").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_CONTEXT_SIZE").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_PIX_FMT_DESC").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_AV_REVERSE").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_AUDIOCONVERT").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_CPU_FLAG_MMX2").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_LLS_PRIVATE").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_AVFRAME_LAVC").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_VDPAU").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_GET_CHANNEL_LAYOUT_COMPAT").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_XVMC").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_OPT_TYPE_METADATA").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_DLOG").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_HMAC").unwrap();
	feature("libavutil/avutil.h", None, "FF_API_VAAPI").unwrap();

	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_VIMA_DECODER").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_REQUEST_CHANNELS").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_OLD_DECODE_AUDIO").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_OLD_ENCODE_AUDIO").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_OLD_ENCODE_VIDEO").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_CODEC_ID").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_AUDIO_CONVERT").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_AVCODEC_RESAMPLE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_DEINTERLACE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_DESTRUCT_PACKET").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_GET_BUFFER").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_MISSING_SAMPLE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_LOWRES").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_CAP_VDPAU").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_BUFS_VDPAU").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_VOXWARE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_SET_DIMENSIONS").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_DEBUG_MV").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_AC_VLC").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_OLD_MSMPEG4").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_ASPECT_EXTENDED").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_THREAD_OPAQUE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_CODEC_PKT").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_ARCH_ALPHA").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_XVMC").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_ERROR_RATE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_QSCALE_TYPE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_MB_TYPE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_MAX_BFRAMES").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_NEG_LINESIZES").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_EMU_EDGE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_ARCH_SH4").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_ARCH_SPARC").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_UNUSED_MEMBERS").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_IDCT_XVIDMMX").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_INPUT_PRESERVED").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_NORMALIZE_AQP").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_GMC").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_MV0").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_CODEC_NAME").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_AFD").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_VISMV").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_DV_FRAME_PROFILE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_AUDIOENC_DELAY").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_VAAPI_CONTEXT").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_AVCTX_TIMEBASE").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_MPV_OPT").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_STREAM_CODEC_TAG").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_QUANT_BIAS").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_RC_STRATEGY").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_CODED_FRAME").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_MOTION_EST").unwrap();
	feature("libavcodec/avcodec.h", Some("avcodec"), "FF_API_WITHOUT_PREFIX").unwrap();

	feature("libavformat/avformat.h", Some("avformat"), "FF_API_LAVF_BITEXACT").unwrap();
	feature("libavformat/avformat.h", Some("avformat"), "FF_API_LAVF_FRAC").unwrap();
	feature("libavformat/avformat.h", Some("avformat"), "FF_API_URL_FEOF").unwrap();
	feature("libavformat/avformat.h", Some("avformat"), "FF_API_PROBESIZE_32").unwrap();

	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_AVFILTERPAD_PUBLIC").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_FOO_COUNT").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_AVFILTERBUFFER").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_OLD_FILTER_OPTS").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_OLD_FILTER_OPTS_ERROR").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_AVFILTER_OPEN").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_OLD_FILTER_REGISTER").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_OLD_GRAPH_PARSE").unwrap();
	feature("libavfilter/avfilter.h", Some("avfilter"), "FF_API_NOCONST_GET_NAME").unwrap();

	feature("libavresample/avresample.h", Some("avresample"), "FF_API_RESAMPLE_CLOSE_OPEN").unwrap();

	feature("libswscale/swscale.h", Some("swscale"), "FF_API_SWS_CPU_CAPS").unwrap();
	feature("libswscale/swscale.h", Some("swscale"), "FF_API_ARCH_BFIN").unwrap();
}
