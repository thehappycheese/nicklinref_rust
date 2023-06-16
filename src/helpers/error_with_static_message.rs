use std::{error::Error, fmt};
use warp::{reject::Reject, Rejection};

/// This is a basic Error type which
///
/// 1. Contains a static str message
/// 2. `impl std::error::Error`
/// 2. `impl warp::reject::Reject`
#[derive(Debug)]
pub struct ErrorWithStaticMessage {
    message: &'static str,
}

impl ErrorWithStaticMessage {
    pub fn new(msg: &'static str) -> ErrorWithStaticMessage {
        ErrorWithStaticMessage { message: msg }
    }
    pub fn reject(msg: &'static str) -> Rejection {
        warp::reject::custom(ErrorWithStaticMessage { message: msg })
    }
    pub fn as_rejection(self) -> warp::Rejection {
        warp::reject::custom(self)
    }
    pub fn get_message(&self) -> &str {
        self.message
    }
}

impl Error for ErrorWithStaticMessage {}

impl Reject for ErrorWithStaticMessage {}

impl fmt::Display for ErrorWithStaticMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", &self.message)
    }
}
