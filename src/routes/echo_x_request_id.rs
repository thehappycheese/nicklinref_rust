use warp::{
    http::HeaderValue,
    reply::Response,
    Filter, Rejection, Reply,
};

/// Echoes back the `x-request-id` header from the request, if it is present and
/// can be parsed as a `u64`. Otherwise the response is not modified.
/// 
/// This wrapper can only work on successful requests. To guarantee it happens
/// on rejected requests, the `filter` must have its own custom rejection
/// handler which has already handled all rejections.
///
/// # Example
///
/// ```
/// use warp::{
///     Filter,
///     reply::Reply,
///     http::StatusCode,
///     wrap_fn,
/// };
///
/// let filter = warp::any()
///     .map(|| StatusCode::OK)
///     .with(wrap_fn(echo_x_request_id));
/// ```
pub fn echo_x_request_id<F, T>(
    filter: F,
) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone + Send + Sync + 'static
where
    F: Filter<Extract = (T,), Error = Rejection> + Clone + Send + Sync + 'static,
    T: Reply,
{
    static HEADER: &str = "x-request-id";
    warp::any()
        .and(
            warp::header::optional::<u64>(HEADER) // only echo if valid u64
                .or(warp::any().map(|| None)) // prevent invalid header rejection
                .unify(),
        )
        .and(
            filter
            .map(|reply:T| reply.into_response())
        )
        .map(move |id: Option<u64>, reply: Response| {
            let mut response = reply.into_response();
            if let Some(id) = id {
                response.headers_mut().insert(HEADER, HeaderValue::from(id));
            }
            response
        })
}




#[cfg(test)]
mod tests {
    use warp::{
        http::Response,
        hyper::{
            Body
        },
        Filter,
        Rejection,
        wrap_fn
    };

    use crate::{helpers::ErrorWithStaticMessage, routes};

    macro_rules! create_wrapped_filter_success {
        () => {
            warp::get()
                .map(|| warp::reply())
                .with(warp::wrap_fn(echo_x_request_id))
        };
    }

    use super::*;

    #[tokio::test]
    async fn echo_x_request_id_success_no_header() {
        let wrapped_filter = create_wrapped_filter_success!();
        let response = warp::test::request().filter(&wrapped_filter).await.unwrap();
        assert!(!response.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn echo_x_request_id_success_valid_header() {
        let wrapped_filter = create_wrapped_filter_success!();
        let response = warp::test::request()
            .header("x-request-id", "10")
            .filter(&wrapped_filter)
            .await
            .unwrap();
        assert!(response.headers().get("x-request-id").unwrap() == "10");
    }

    #[tokio::test]
    async fn echo_x_request_id_success_invalid_header() {
        let wrapped_filter = create_wrapped_filter_success!();
        let response = warp::test::request()
            .header("x-request-id", "nux")
            .filter(&wrapped_filter)
            .await
            .unwrap();
        assert!(!response.headers().contains_key("x-request-id"));
    }


    #[tokio::test]
    /// Oh boy was it ever hard to make this test pass
    /// we want the header to be attached regardless of the success of the main 
    /// program.
    /// 
    /// It turns out that is not actually possible without introducing a custom
    /// rejection handler; `.recover(routes::custom_rejection_handler)`
    async fn echo_x_request_id_reject_valid_header_reduced() {
        let wrapped_filter = warp::get()
            .and_then(|| async {
                let result: Result<Response<Body>, Rejection> =
                    Err(ErrorWithStaticMessage::new("Failure Message").into());
                result
            })
            .recover(routes::custom_rejection_handler)
            .with(wrap_fn(echo_x_request_id));

        let response = warp::test::request()
            .header("x-request-id", "10")
            .filter(&wrapped_filter)
            .await
            .unwrap();

        assert_eq!(response.headers().get("x-request-id").unwrap(), "10");
    }

}
