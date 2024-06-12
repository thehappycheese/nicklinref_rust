use crate::helpers::ErrorWithStaticMessage;
use warp::{
    http::StatusCode, reject::{InvalidQuery, MethodNotAllowed, UnsupportedMediaType}, reply::Response, Rejection, Reply
};

pub async fn custom_rejection_handler(rejection: Rejection) -> Result<Response, Rejection> {
    
    let code;
    let message: String;
    
    if rejection.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found".to_owned();
    } else if let Some(custom_reject) = rejection.find::<ErrorWithStaticMessage>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = custom_reject.get_message().to_owned();
    } else if let Some(custom_reject) = rejection.find::<InvalidQuery>() {
        code = StatusCode::BAD_REQUEST;
        message = custom_reject.to_string();
    } else if let Some(custom_reject) = rejection.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = custom_reject.to_string();
    } else if let Some(custom_reject) = rejection.find::<UnsupportedMediaType>(){
        code = StatusCode::UNSUPPORTED_MEDIA_TYPE;
        message = custom_reject.to_string();
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Unexpected error".to_owned();
    }

    let res = warp::http::Response::builder()
        .status(code)
        .body(message)
        .unwrap();

    let result: Result<_, Rejection> = Ok(res.into_response());
    result
}
