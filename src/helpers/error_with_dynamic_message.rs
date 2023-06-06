use std::{error::Error, fmt};

use warp::{reject::Reject};

#[derive(Debug)]
pub struct ErrorWithDynamicMessage{
	msg:String
}

impl ErrorWithDynamicMessage{
	pub fn new(msg:String)->ErrorWithDynamicMessage{
		ErrorWithDynamicMessage{msg:msg}
	}
}

impl Error for ErrorWithDynamicMessage {}

impl Reject for ErrorWithDynamicMessage {}

impl fmt::Display for ErrorWithDynamicMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", &self.msg)
	}
}