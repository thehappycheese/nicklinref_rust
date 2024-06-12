use std::sync::Arc;

use warp::{Filter, Rejection};

use crate::data::IndexedData;

use super::{
    geoprocessing::{get_linestring, get_linestring_m},
    query_parameters::QueryParametersLine,
    with_shared_data
};

pub fn lines(
    indexed_data: Arc<IndexedData>,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::path::end()
    .and(with_shared_data(indexed_data.clone()))
    .and(
        warp::get().and(warp::query())
        .or(warp::post().and(warp::body::json()))
        .unify()
    )
    .and_then(|
        indexed_data: Arc<IndexedData>,
        query: QueryParametersLine
    | async move {
        if query.m {
            get_linestring_m(&query, &indexed_data).map_err(|err|err.as_rejection())
        } else {
            get_linestring(&query, &indexed_data).map_err(|err|err.as_rejection())
        }
    })
}
