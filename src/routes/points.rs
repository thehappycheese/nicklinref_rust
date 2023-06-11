use std::sync::Arc;

use warp::Filter;

use crate::{
    helpers::with_shared_data,
    data::IndexedData,
};

use super::{
    geoprocessing::get_points,
    query_parameters::QueryParametersPoint
};

pub fn points(
    indexed_data: Arc<IndexedData>
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::get()
    .and(with_shared_data(indexed_data.clone()))
    .and(warp::query())
    .and_then(|
            indexed_data: Arc<IndexedData>,
            query: QueryParametersPoint
        | async move {
            get_points(&query, &indexed_data).map_err(|err|err.as_rejection())
        })
}
