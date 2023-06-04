use std::{error::Error, fmt};
use warp::{reject::Reject, Rejection};


/// This is a basic Error type which
/// 
/// 1. Contains a static str message
/// 2. `impl std::error::Error`
/// 2. `impl warp::reject::Reject`
#[derive(Debug)]
pub struct ErrorWithMessage{
	msg:& 'static str
}

impl ErrorWithMessage{
	pub fn new(msg:& 'static str)->ErrorWithMessage{
		ErrorWithMessage{msg:msg}
	}
	pub fn reject(msg:& 'static str) -> Rejection{
		warp::reject::custom(ErrorWithMessage{msg:msg})
	}
}

impl Error for ErrorWithMessage {}

impl Reject for ErrorWithMessage {}

impl fmt::Display for ErrorWithMessage {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "BasicError({})", &self.msg)
	}
}