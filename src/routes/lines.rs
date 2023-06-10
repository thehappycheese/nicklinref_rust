use std::sync::Arc;

use warp::Filter;

use crate::{
    data::esri_serde::LayerSaved,
    helpers::{with_shared_data, ErrorWithStaticMessage},
    data::{index::LookupMap, IndexedData},
};

use super::{
    geoprocessing::{get_linestring, get_linestring_m},
    query_parameters::QueryParametersLine
};

pub fn lines(
    indexed_data: Arc<IndexedData>,
    
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::get()
    .and(with_shared_data(indexed_data.clone()))
    .and(warp::query())
    .and_then(|
            indexed_data: Arc<IndexedData>,
            query: QueryParametersLine
        | async move {
            if query.m {
                get_linestring_m(&query, &indexed_data).into()
            } else {
                get_linestring(&query, &indexed_data)
            }
        })
}
