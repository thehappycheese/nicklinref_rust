

use std::sync::Arc;

use warp::Filter;

use crate::{
    helpers::ErrorWithStaticMessage,
    data::IndexedData,
};

use super::{
    geoprocessing::get_linestring,
    query_parameters::QueryParameterBatch,
    with_shared_data
};

pub fn lines_batch(
    indexed_data:Arc<IndexedData>
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::post()
    .and(warp::path("batch").and(warp::path::end()))
    .and(with_shared_data(indexed_data.clone()))
    .and(warp::body::bytes())
    .and_then(|
            indexed_data: Arc<IndexedData>,
            body: bytes::Bytes
        | async move {
            if let Ok(batch_query) = QueryParameterBatch::try_from(body) {
                let result_string = 
                    batch_query
                    .0
                    .iter()
                    .map(|query| match get_linestring(query, &indexed_data) {
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

