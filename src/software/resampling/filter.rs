use ffi::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Filter {
	Cubic,
	BlackmanNuttall,
	Kaiser,
}

impl From<SwrFilterType> for Filter {
	fn from(value: SwrFilterType) -> Filter {
		match value {
    	SWR_FILTER_TYPE_CUBIC            => Filter::Cubic,
    	SWR_FILTER_TYPE_BLACKMAN_NUTTALL => Filter::BlackmanNuttall,
    	SWR_FILTER_TYPE_KAISER           => Filter::Kaiser,
		}
	}
}

impl Into<SwrFilterType> for Filter {
	fn into(self) -> SwrFilterType {
		match self {
    	Filter::Cubic          => SWR_FILTER_TYPE_CUBIC,
    	Filter::BlackmanNuttall => SWR_FILTER_TYPE_BLACKMAN_NUTTALL,
    	Filter::Kaiser         => SWR_FILTER_TYPE_KAISER,
		}
	}
}
