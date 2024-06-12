use std::sync::Arc;

use serde::Deserialize;
use warp::Filter;

use crate::{data::IndexedData, filters::geoprocessing::get_linestring};
use super::{
    geoprocessing::get_points,
    query_parameters::QueryParametersUnified,
    with_shared_data
};






pub fn unified_batch(
    indexed_data: Arc<IndexedData>
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::path("batch2").and(warp::path::end())
    .and(with_shared_data(indexed_data.clone()))
    .and(warp::post().and(warp::body::json()))
    .and_then(|
        indexed_data: Arc<IndexedData>,
        query: Vec<QueryParametersUnified>
    | async move {
        use QueryParametersUnified::*;
        // TODO: must not be used with non JSON return types... or those must be handled differently?
        // todo: could slap rayon in here for some easy parallelization perhaps?
        let results:Vec<String> = query.iter().map(|request| match request {
            Point(point_request)=>get_points(&point_request, &indexed_data).unwrap_or("null".to_owned()),
            Line (line_request)  =>get_linestring(&line_request, &indexed_data).unwrap_or("null".to_owned()),
        }).collect();
        Ok::<std::string::String, warp::Rejection>(format!("[{}]",results.join(",")))
    })
}


