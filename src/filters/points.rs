use std::{collections::HashMap, sync::Arc};

use warp::Filter;

use crate::data::IndexedData;

use super::{
    geoprocessing::{
        get_points,
        get_linestring,
        get_linestring_m
    },
    query_parameters::QueryParametersUnified,
    with_shared_data
};

pub fn points(
    indexed_data: Arc<IndexedData>
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::path::end()
    .and(with_shared_data(indexed_data.clone()))
    .and(
        warp::get().and(warp::query())
        .or(warp::post().and(warp::body::json()))
        .unify()
    )
    .and_then(|
        indexed_data: Arc<IndexedData>,
        query: QueryParametersUnified
    | async move {
        use QueryParametersUnified::*;
        match query {
            Point(point_request) => get_points    (&point_request, &indexed_data).map_err(|err|err.as_rejection()),
            Line (line_request)  => if line_request.m {
                get_linestring_m(&line_request , &indexed_data).map_err(|err|err.as_rejection())
            }else{
                get_linestring(&line_request , &indexed_data).map_err(|err|err.as_rejection())
            },
        }
    })
}
