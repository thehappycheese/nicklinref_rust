
use warp::{Filter, reply::Response};

pub fn echo_header_x_request_id<FilterType, ExtractType>(
    filter: FilterType
) -> impl Filter<Extract = (&'static str,)> + Clone + Send + Sync + 'static
where
    FilterType: Filter<
        Extract = (ExtractType,),
        Error = std::convert::Infallible
    > + Clone + Send + Sync + 'static,
    FilterType::Extract: warp::Reply,
{
    warp::any()
        .and(filter)
        .and(warp::header::<i64>("x-request-id"))
        .map(|something:warp::Reply /*???*/, id:i64|{
            // inject the header???
            todo!()
        }).recover(|something|something)
}


fn hello_wrapper<F, T>(
    filter: F,
) -> impl Filter<Extract = (&'static str,)> + Clone + Send + Sync + 'static
where
    F: Filter<Extract = (T,), Error = std::convert::Infallible> + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    warp::any()
        .and(filter)
        .map(|_arg| "wrapped hello world")
}