use std::{error, fmt};
use fltk::prelude::FltkError;
use jpeg_encoder::EncodingError;

use crate::processing::Halted;

#[derive(Debug, Clone)]
pub struct MyError {
	msg: String
}

impl fmt::Display for MyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.msg)
	}
}

impl error::Error for MyError { }

impl MyError {
	pub fn new(msg: String) -> Self {
		MyError {
			msg
		}
	}

	pub fn get_message(&self) -> String {
		self.msg.clone()
	}
}

impl From<std::io::Error> for MyError { 
	fn from(err: std::io::Error) -> Self {
		MyError {
			msg: err.to_string()
		}
	}
}

impl From<FltkError> for MyError { 
	fn from(err: FltkError) -> Self {
		MyError {
			msg: err.to_string()
		}
	}
}

impl From<EncodingError> for MyError { 
	fn from(err: EncodingError) -> Self {
		MyError {
			msg: err.to_string()
		}
	}
}

impl From<Halted> for MyError { 
	fn from(_: Halted) -> Self {
		MyError {
			msg: "Задача была прервана".to_string()
		}
	}
}