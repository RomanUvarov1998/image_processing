use std::{
	error,
	string,
	fmt
};
use fltk::prelude::*;

#[derive(Debug, Clone)]
pub struct MyError {
	msg: String
}

impl fmt::Display for MyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "MyError occured: {}", self.msg)
	}
}

impl error::Error for MyError { }

impl MyError {
	pub fn new(msg: String) -> Self {
		MyError {
			msg
		}
	}
}

impl From<string::FromUtf8Error> for MyError { 
	fn from(err: string::FromUtf8Error) -> Self {
		MyError {
			msg: err.to_string()
		}
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

impl From<image::ImageError> for MyError { 
	fn from(err: image::ImageError) -> Self {
		MyError {
			msg: err.to_string()
		}
	}
}