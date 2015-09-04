use std::ptr;
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use ffi::*;

pub enum Format {
	Input(Input),
	Output(Output),
}

impl Format {
	pub fn name(&self) -> &str {
		match self {
			&Format::Input(ref f)  => f.name(),
			&Format::Output(ref f) => f.name()
		}
	}

	pub fn description(&self) -> &str {
		match self {
			&Format::Input(ref f)  => f.description(),
			&Format::Output(ref f) => f.description()
		}
	}

	pub fn extensions(&self) -> Vec<&str> {
		match self {
			&Format::Input(ref f)  => f.extensions(),
			&Format::Output(ref f) => f.extensions()
		}
	}

	pub fn mime_types(&self) -> Vec<&str> {
		match self {
			&Format::Input(ref f)  => f.mime_types(),
			&Format::Output(ref f) => f.mime_types()
		}
	}
}

pub struct Input {
	ptr: *mut AVInputFormat,
}

impl Input {
	pub unsafe fn wrap(ptr: *mut AVInputFormat) -> Self {
		Input { ptr: ptr }
	}

	pub unsafe fn as_ptr(&self) -> *const AVInputFormat {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVInputFormat {
		self.ptr
	}
}

impl Input {
	pub fn name(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).name).to_bytes())
		}
	}

	pub fn description(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).name).to_bytes())
		}
	}

	pub fn extensions(&self) -> Vec<&str> {
		unsafe {
			let ptr = (*self.as_ptr()).extensions;

			if ptr.is_null() {
				Vec::new()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}

	pub fn mime_types(&self) -> Vec<&str> {
		unsafe {
			let ptr = (*self.as_ptr()).mime_type;

			if ptr.is_null() {
				Vec::new()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}
}

pub struct Output {
	ptr: *mut AVOutputFormat,
}

impl Output {
	pub unsafe fn wrap(ptr: *mut AVOutputFormat) -> Self {
		Output { ptr: ptr }
	}

	pub unsafe fn as_ptr(&self) -> *const AVOutputFormat {
		self.ptr as *const _
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AVOutputFormat {
		self.ptr
	}
}

impl Output {
	pub fn name(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).name).to_bytes())
		}
	}

	pub fn description(&self) -> &str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr((*self.as_ptr()).name).to_bytes())
		}
	}

	pub fn extensions(&self) -> Vec<&str> {
		unsafe {
			let ptr = (*self.as_ptr()).extensions;

			if ptr.is_null() {
				Vec::new()
			}
			else {
				from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()).split(',').collect()
			}
		}
	}

	pub fn mime_types(&self) -> Vec<&str> {
		unsafe {
			let ptr = (*self.as_ptr()).mime_type;

			if ptr.is_null() {
				Vec::new()
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
	step:   Step,
}

enum Step {
	Input,
	Output,
	Done,
}

impl FormatIter {
	pub fn new() -> Self {
		FormatIter { input: ptr::null_mut(), output: ptr::null_mut(), step: Step::Input }
	}
}

impl Iterator for FormatIter {
	type Item = Format;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		unsafe {
			match self.step {
				Step::Input => {
					let ptr = av_iformat_next(self.input);

					if ptr.is_null() && !self.input.is_null() {
						self.step = Step::Output;

						self.next()
					}
					else {
						self.input = ptr;

						Some(Format::Input(Input::wrap(ptr)))
					}
				}

				Step::Output => {
					let ptr = av_oformat_next(self.output);

					if ptr.is_null() && !self.output.is_null() {
						self.step = Step::Done;

						self.next()
					}
					else {
						self.output = ptr;

						Some(Format::Output(Output::wrap(ptr)))
					}
				}

				Step::Done =>
					None
			}
		}
	}
}
