use ffi::*;
use ::{Rational, Rounding};

pub const DEFAULT: Rational = Rational(AV_TIME_BASE_Q.num, AV_TIME_BASE_Q.den);

pub trait Rescaling {
	fn rescale<S, D>(&self, source: S, destination: D) -> i64
		where S: Into<Rational>,
		      D: Into<Rational>;

	fn rescale_with<S, D>(&self, source: S, destination: D, rounding: Rounding) -> i64
		where S: Into<Rational>,
		      D: Into<Rational>;
}

impl<T: Into<i64> + From<i64>> Rescaling for T {
	fn rescale<S, D>(&self, source: S, destination: D) -> i64
		where S: Into<Rational>,
		      D: Into<Rational>
	{
		unsafe {
			av_rescale_q(self.into(), source.into().into(), destination.into().into()).into()
		}
	}

	fn rescale_with<S, D>(&self, source: S, destination: D, rounding: Rounding) -> i64
		where S: Into<Rational>,
		      D: Into<Rational>
	{
		unsafe {
			av_rescale_q(self.into(), source.into().into(), destination.into().into()).into()
		}
	}
}
