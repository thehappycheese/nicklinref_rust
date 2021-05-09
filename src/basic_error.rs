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


// This next line explains why all this non-sense custom error implementation is necessary;
// Reject is just a list of constraints which make this error type threadsafe.
// So annoying to have to write all this out to emit custom errors.
// Maybe we can convert BasicErrorWarp into an enum so we can have a few different errors without all this boilerplate for each?
impl Reject for BasicErrorWarp{}

impl fmt::Display for BasicErrorWarp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "BasicError({})", &self.msg)
	}
}