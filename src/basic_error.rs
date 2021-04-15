use std::{error::Error, fmt};
use warp::reject::Reject;
#[derive(Debug)]
pub struct BasicError{
	msg:& 'static str
}

impl BasicError{
	pub fn new(msg:& 'static str)->BasicError{
		BasicError{msg:msg}
	}
}

impl Error for BasicError {}

impl fmt::Display for BasicError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "BasicError({})", &self.msg)
	}
}
////////////////////////
#[derive(Debug)]
pub struct BasicErrorWarp{
	pub msg:& 'static str
}

impl BasicErrorWarp{
	pub fn new(msg:& 'static str)->BasicErrorWarp{
		BasicErrorWarp{msg:msg}
	}
}

impl Error for BasicErrorWarp {}

impl Reject for BasicErrorWarp{}

impl fmt::Display for BasicErrorWarp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "BasicError({})", &self.msg)
	}
}