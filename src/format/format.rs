use std::ptr;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

pub enum Format {
	Input(Input),
	Output(Output),
}

impl Format {
	pub fn name<'a>(&'a self) -> &'a str {
		match self {
			&Format::Input(ref f)  => f.name(),
			&Format::Output(ref f) => f.name()
		}
	}

	pub fn description<'a>(&'a self) -> &'a str {
		match self {
			&Format::Input(ref f)  => f.description(),
			&Format::Output(ref f) => f.description()
		}
	}

	pub fn extensions<'a>(&'a self) -> Vec<&'a str> {
		match self {
			&Format::Input(ref f)  => f.extensions(),
			&Format::Output(ref f) => f.extensions()
		}
	}

	pub fn mime_types<'a>(&'a self) -> Vec<&'a str> {
		match self {
			&Format::Input(ref f)  => f.mime_types(),
			&Format::Output(ref f) => f.mime_types()
		}
	}
}

pub struct Input {
	pub ptr: *mut AVInputFormat,
}

impl Input {
	pub fn wrap(ptr: *mut AVInputFormat) -> Self {
		Input { ptr: ptr }
	}

	pub fn name<'a>(&'a self) -> &'a str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).name).to_bytes())
		}
	}

	pub fn description<'a>(&'a self) -> &'a str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).name).to_bytes())
		}
	}

	pub fn extensions<'a>(&'a self) -> Vec<&'a str> {
		unsafe {
			let ptr = (*self.ptr).extensions;

			if ptr == ptr::null() {
				vec!()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}

	pub fn mime_types<'a>(&'a self) -> Vec<&'a str> {
		unsafe {
			let ptr = (*self.ptr).mime_type;

			if ptr == ptr::null() {
				vec!()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}
}

pub struct Output {
	pub ptr: *mut AVOutputFormat,
}

impl Output {
	pub fn wrap(ptr: *mut AVOutputFormat) -> Self {
		Output { ptr: ptr }
	}

	pub fn name<'a>(&'a self) -> &'a str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).name).to_bytes())
		}
	}

	pub fn description<'a>(&'a self) -> &'a str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.ptr).name).to_bytes())
		}
	}

	pub fn extensions<'a>(&'a self) -> Vec<&'a str> {
		unsafe {
			let ptr = (*self.ptr).extensions;

			if ptr == ptr::null() {
				vec!()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}

	pub fn mime_types<'a>(&'a self) -> Vec<&'a str> {
		unsafe {
			let ptr = (*self.ptr).mime_type;

			if ptr == ptr::null() {
				vec!()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}
}

pub fn list() -> FormatIter {
	FormatIter::new()
}

pub struct FormatIter {
	input:  *mut AVInputFormat,
	output: *mut AVOutputFormat,
	step:   usize,
}

impl FormatIter {
	pub fn new() -> Self {
		FormatIter { input: ptr::null_mut(), output: ptr::null_mut(), step: 0 }
	}
}

impl Iterator for FormatIter {
	type Item = Format;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			match self.step {
				0 => {
					let ptr = av_iformat_next(self.input);

					if ptr == ptr::null_mut() && self.input != ptr::null_mut() {
						self.step = 1;

						self.next()
					}
					else {
						self.input = ptr;

						Some(Format::Input(Input::wrap(ptr)))
					}
				},

				1 => {
					let ptr = av_oformat_next(self.output);

					if ptr == ptr::null_mut() && self.output != ptr::null_mut() {
						self.step = 2;

						self.next()
					}
					else {
						self.output = ptr;

						Some(Format::Output(Output::wrap(ptr)))
					}
				},

				_ => None
			}
		}
	}
}
