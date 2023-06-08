use std::sync::Arc;

use warp::Filter;

use crate::{
    data::esri_serde::LayerSaved,
    helpers::{with_shared_data, ErrorWithStaticMessage},
    data::index::LookupMap,
};

use super::{
    geoprocessing::{get_linestring, get_linestring_m},
    query_parameters::QueryParametersLine
};

pub fn lines(
    data: Arc<LayerSaved>,
    data_index: Arc<LookupMap>,
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::get()
    .and(with_shared_data(data.clone()))
    .and(with_shared_data(data_index.clone())).clone()
    .and(warp::query())
    .and_then(|
            data: Arc<LayerSaved>,
            data_index: Arc<LookupMap>,
            query: QueryParametersLine
        | async move {
            if query.m {
                match get_linestring_m(&query, &data, &data_index) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(ErrorWithStaticMessage::reject(e)),

                }
            } else {
                match get_linestring(&query, &data, &data_index) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(ErrorWithStaticMessage::reject(e)),
                }
            }
        })
}
