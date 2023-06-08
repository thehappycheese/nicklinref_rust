

use std::sync::Arc;

use warp::Filter;

use crate::{
    data::esri_serde::LayerSaved,
    helpers::{with_shared_data, ErrorWithStaticMessage},
    data::index::LookupMap,
};

use super::{
    geoprocessing::{get_linestring},
    query_parameters::QueryParameterBatch
};

pub fn lines_batch(
    data: Arc<LayerSaved>,
    data_index: Arc<LookupMap>,
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::post()
    .and(warp::path("batch").and(warp::path::end()))
    .and(with_shared_data(data.clone()))
    .and(with_shared_data(data_index.clone()))
    .and(warp::body::bytes())
    .and_then(|
            data: Arc<LayerSaved>,
            data_index: Arc<LookupMap>,
            body: bytes::Bytes
        | async move {
            if let Ok(batch_query) = QueryParameterBatch::try_from(body) {
                let result_string = 
                    batch_query
                    .0
                    .iter()
                    .map(|query| match get_linestring(query, &data, &data_index) {
                        Ok(x) => x,
                        Err(_) => "null".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(",");
                Ok(format!("[{}]", result_string))
            } else {
                Err(ErrorWithStaticMessage::reject("Unable to parse batch query parameters"))
            }
        })
}

