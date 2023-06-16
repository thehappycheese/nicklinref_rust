use std::{error::Error, sync::Arc};

use warp::{Filter, wrap_fn, filters::BoxedFilter, reply::Response, fs::File, Reply};

use crate::{data::IndexedData, settings::Settings};

pub async fn load_data_and_get_combined_routes(settings:&Settings) -> Result<BoxedFilter<(Response,)>, Box<dyn Error>> {

    // Load data
    let indexed_data:Arc<_> = IndexedData::load(
        &settings.NLR_DATA_FILE,
        &settings.NLR_DATA_SOURCE_URL,
        &settings.NLR_FORCE_UPDATE_DATA
    ).await?.into();

    let route_static_folder = warp::fs::dir(settings.NLR_STATIC_HTTP.clone());
    let route_show = warp::path("show").and(route_static_folder);
    let route_lines = super::lines(indexed_data.clone());
    let route_points = super::points(indexed_data.clone());
    let route_lines_batch = super::lines_batch(indexed_data.clone());

    let x = route_show.map(|r:File| r.into_response()).or(
        route_lines
        .or(route_points)
        .or(
            route_lines_batch
            .with(warp::compression::gzip())
        )
        .recover(super::custom_rejection_handler)
        .with(wrap_fn(super::echo_x_request_id))
    ).unify();
    Ok(x.boxed())
}
