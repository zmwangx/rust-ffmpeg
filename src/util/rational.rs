use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Div};
use std::fmt;

use libc::{c_int, int64_t};
use ffi::*;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Rational(pub AVRational);

impl Rational {
	pub fn new(numerator: i32, denominator: i32) -> Self {
		Rational(AVRational { num: numerator as c_int, den: denominator as c_int })
	}

	pub fn numerator(&self) -> i32 {
		self.0.num as i32
	}

	pub fn denominator(&self) -> i32 {
		self.0.den as i32
	}

	pub fn reduce(&self, max: i32) -> (Rational, bool) {
		unsafe {
			let mut dst_num: c_int = 0;
			let mut dst_den: c_int = 0;

			let exact = av_reduce(&mut dst_num, &mut dst_den,
			                      self.numerator() as int64_t, self.denominator() as int64_t,
			                      max as int64_t);

			(Rational::new(dst_num, dst_den), exact == 1)
		}
	}

	pub fn invert(&self) -> Rational {
		unsafe {
			Rational(av_inv_q(self.0))
		}
	}
}

impl fmt::Display for Rational {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.write_str(&format!("{}/{}", self.numerator(), self.denominator()))
	}
}

impl fmt::Debug for Rational {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.write_str(&format!("Rational({}/{})", self.numerator(), self.denominator()))
	}
}

impl From<f64> for Rational {
	fn from(value: f64) -> Rational {
		unsafe {
			Rational(av_d2q(value, c_int::max_value()))
		}
	}
}

impl Into<f64> for Rational {
	fn into(self) -> f64 {
		unsafe {
			av_q2d(self.0)
		}
	}
}

impl PartialOrd for Rational {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		unsafe {
			match av_cmp_q(self.0, other.0) {
				 0 => Some(Ordering::Equal),
				 1 => Some(Ordering::Greater),
				-1 => Some(Ordering::Less),

				_ => None
			}
		}
	}
}

impl Add for Rational {
	type Output = Rational;

	fn add(self, other: Rational) -> Rational {
		unsafe {
			Rational(av_add_q(self.0, other.0))
		}
	}
}

impl Sub for Rational {
	type Output = Rational;

	fn sub(self, other: Rational) -> Rational {
		unsafe {
			Rational(av_sub_q(self.0, other.0))
		}
	}
}

impl Mul for Rational {
	type Output = Rational;

	fn mul(self, other: Rational) -> Rational {
		unsafe {
			Rational(av_mul_q(self.0, other.0))
		}
	}
}

impl Div for Rational {
	type Output = Rational;

	fn div(self, other: Rational) -> Rational {
		unsafe {
			Rational(av_div_q(self.0, other.0))
		}
	}
}

pub fn nearer(q: Rational, q1: Rational, q2: Rational) -> Ordering {
	unsafe {
		match av_nearer_q(q.0, q1.0, q2.0) {
			 1 => Ordering::Greater,
			-1 => Ordering::Less,
			 _ => Ordering::Equal,
		}
	}
}
