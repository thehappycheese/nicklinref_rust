use warp::{http::HeaderValue, reply::Response, Filter, Rejection, Reply};

/// Echoes back the `x-request-id` header from the request, if it is present and
/// can be parsed as a `u64`. Otherwise the response is not modified.
/// 
/// TODO: the desired behavior is that if the header is present and valid it
/// will be echoed even if the response from `filter` is a rejection.
/// Currently I don't see a way to accomplish that...
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
        .and(filter)
        .map(move |id: Option<u64>, reply: T| {
            let mut response = reply.into_response();
            // note response status is not checked; the header will be
            // echo-ed even on a rejection
            if let Some(id) = id {
                response.headers_mut().insert(HEADER, HeaderValue::from(id));
            }
            response
        })
}
