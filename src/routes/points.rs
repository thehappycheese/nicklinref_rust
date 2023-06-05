use std::sync::Arc;

use warp::Filter;

use crate::{
    data::esri_serde::LayerSaved,
    helpers::{with_shared_data, ErrorWithMessage},
    data::index::LookupMap,
};

use super::{
    geoprocessing::get_points,
    query_parameters::QueryParametersPoint
};

pub fn points(
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
            query: QueryParametersPoint
        | async move {
            match get_points(&query, &data, &data_index) {
                Ok(s) => Ok(s),
                Err(e) => Err(ErrorWithMessage::reject(e)),
            }
        })
}
