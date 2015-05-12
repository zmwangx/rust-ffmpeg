use ffi::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Rational(pub AVRational);

impl Rational {
	pub fn numerator(&self) -> i32 {
		self.0.num
	}

	pub fn denominator(&self) -> i32 {
		self.0.den
	}
}
