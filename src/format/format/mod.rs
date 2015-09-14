pub mod flag;
pub use self::flag::Flags;

mod input;
pub use self::input::Input;

mod output;
pub use self::output::Output;

mod iter;
pub use self::iter::Iter;

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

pub fn list() -> Iter {
	Iter::new()
}
