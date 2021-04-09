use std::{error::Error, fmt};

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