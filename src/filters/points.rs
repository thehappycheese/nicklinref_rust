use std::sync::Arc;

use warp::Filter;

use crate::data::IndexedData;

use super::{
    geoprocessing::get_points,
    query_parameters::QueryParametersPoint,
    with_shared_data
};

pub fn points(
    indexed_data: Arc<IndexedData>
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::path::end()
    .and(warp::get())
    .and(with_shared_data(indexed_data.clone()))
    .and(warp::query())
    .and_then(|
        indexed_data: Arc<IndexedData>,
        query: QueryParametersPoint
    | async move {
        get_points    (&query, &indexed_data).map_err(|err|err.as_rejection())
    })
    // New version of the endpoint must be descriminated by the `/point` route
    // this new version will accept both GET and POST requests
    .or(
        warp::path("point")
        .and(with_shared_data(indexed_data.clone()))
        .and(
            warp::get().and(warp::query())
            .or(warp::post().and(warp::body::json()))
            .unify()
        )
        .and_then(|
            indexed_data: Arc<IndexedData>,
            query: QueryParametersPoint
        | async move {
            get_points(&query, &indexed_data).map_err(|err|err.as_rejection())
        })
    )
    .unify()
}
