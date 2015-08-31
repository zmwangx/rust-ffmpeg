use libc::c_ulonglong;
use ffi::*;

bitflags! {
	flags ChannelLayout: c_ulonglong {
		const FRONT_LEFT            = AV_CH_FRONT_LEFT,
		const FRONT_RIGHT           = AV_CH_FRONT_RIGHT,
		const FRONT_CENTER          = AV_CH_FRONT_CENTER,
		const LOW_FREQUENCY         = AV_CH_LOW_FREQUENCY,
		const BACK_LEFT             = AV_CH_BACK_LEFT,
		const BACK_RIGHT            = AV_CH_BACK_RIGHT,
		const FRONT_LEFT_OF_CENTER  = AV_CH_FRONT_LEFT_OF_CENTER,
		const FRONT_RIGHT_OF_CENTER = AV_CH_FRONT_RIGHT_OF_CENTER,
		const BACK_CENTER           = AV_CH_BACK_CENTER,
		const SIDE_LEFT             = AV_CH_SIDE_LEFT,
		const SIDE_RIGHT            = AV_CH_SIDE_RIGHT,
		const TOP_CENTER            = AV_CH_TOP_CENTER,
		const TOP_FRONT_LEFT        = AV_CH_TOP_FRONT_LEFT,
		const TOP_FRONT_CENTER      = AV_CH_TOP_FRONT_CENTER,
		const TOP_FRONT_RIGHT       = AV_CH_TOP_FRONT_RIGHT,
		const TOP_BACK_LEFT         = AV_CH_TOP_BACK_LEFT,
		const TOP_BACK_CENTER       = AV_CH_TOP_BACK_CENTER,
		const TOP_BACK_RIGHT        = AV_CH_TOP_BACK_RIGHT,
		const STEREO_LEFT           = AV_CH_STEREO_LEFT,
		const STEREO_RIGHT          = AV_CH_STEREO_RIGHT,
		const WIDE_LEFT             = AV_CH_WIDE_LEFT,
		const WIDE_RIGHT            = AV_CH_WIDE_RIGHT,
		const SURROUND_DIRECT_LEFT  = AV_CH_SURROUND_DIRECT_LEFT,
		const SURROUND_DIRECT_RIGHT = AV_CH_SURROUND_DIRECT_RIGHT,
		const LOW_FREQUENCY_2       = AV_CH_LOW_FREQUENCY_2,
		const NATIVE                = AV_CH_LAYOUT_NATIVE,

		const MONO               = FRONT_CENTER.bits,
		const STEREO             = FRONT_LEFT.bits | FRONT_RIGHT.bits,
		const _2POINT1           = STEREO.bits | LOW_FREQUENCY.bits,
		const _2_1               = STEREO.bits | BACK_CENTER.bits,
		const SURROUND           = STEREO.bits | FRONT_CENTER.bits,
		const _3POINT1           = SURROUND.bits | LOW_FREQUENCY.bits,
		const _4POINT0           = SURROUND.bits | BACK_CENTER.bits,
		const _4POINT1           = _4POINT0.bits | LOW_FREQUENCY.bits,
		const _2_2               = STEREO.bits | SIDE_LEFT.bits | SIDE_RIGHT.bits,
		const QUAD               = STEREO.bits | BACK_LEFT.bits | BACK_RIGHT.bits,
		const _5POINT0           = SURROUND.bits | SIDE_LEFT.bits | SIDE_RIGHT.bits,
		const _5POINT1           = _5POINT0.bits | LOW_FREQUENCY.bits,
		const _5POINT0_BACK      = SURROUND.bits | BACK_LEFT.bits | BACK_RIGHT.bits,
		const _5POINT1_BACK      = _5POINT0_BACK.bits | LOW_FREQUENCY.bits,
		const _6POINT0           = _5POINT0.bits | BACK_CENTER.bits,
		const _6POINT0_FRONT     = _2_2.bits | FRONT_LEFT_OF_CENTER.bits | FRONT_RIGHT_OF_CENTER.bits,
		const HEXAGONAL          = _5POINT0_BACK.bits | BACK_CENTER.bits,
		const _6POINT1           = _5POINT1.bits | BACK_CENTER.bits,
		const _6POINT1_BACK      = _5POINT1_BACK.bits | BACK_CENTER.bits,
		const _6POINT1_FRONT     = _6POINT0_FRONT.bits | LOW_FREQUENCY.bits,
		const _7POINT0           = _5POINT0.bits | BACK_LEFT.bits | BACK_RIGHT.bits,
		const _7POINT0_FRONT     = _5POINT0.bits | FRONT_LEFT_OF_CENTER.bits | FRONT_RIGHT_OF_CENTER.bits,
		const _7POINT1           = _5POINT1.bits | BACK_LEFT.bits | BACK_RIGHT.bits,
		const _7POINT1_WIDE      = _5POINT1.bits | FRONT_LEFT_OF_CENTER.bits | FRONT_RIGHT_OF_CENTER.bits,
		const _7POINT1_WIDE_BACK = _5POINT1_BACK.bits | FRONT_LEFT_OF_CENTER.bits | FRONT_RIGHT_OF_CENTER.bits,
		const OCTAGONAL          = _5POINT0.bits | BACK_LEFT.bits | BACK_CENTER.bits | BACK_RIGHT.bits,
		const STEREO_DOWNMIX     = STEREO_LEFT.bits | STEREO_RIGHT.bits,
	}
}

impl ChannelLayout {
	pub fn channels(&self) -> i32 {
		unsafe {
			av_get_channel_layout_nb_channels(self.bits())
		}
	}
}
