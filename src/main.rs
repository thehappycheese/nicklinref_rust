use std::{sync::Arc, net::SocketAddr};

use warp::{Filter, wrap_fn};

mod helpers;
use helpers::{
    echo_x_request_id
};

mod routes;
mod data;
use data::{
    esri_serde::LayerSaved,
    load_data_from_file,
    download_data,
    index::{
        index_data,
        LookupMap
    },
};

mod settings;
use settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read settings
    let settings:Settings = match Settings::load_settings(){
        Ok(settings)=>settings,
        Err(e)=> return Err(e.into())
    }.into();

    // Load data
    let data: Arc<LayerSaved> = match load_data_from_file(&settings) {
        Ok(response) => response,
        Err(error_message) => {
            println!(
                "Failed to load from cache due to error {}. Will try re-download.",
                error_message
            );
            download_data(&settings).await?
        }
    }.into();
    println!("Loaded {} features.", data.features.len());

    // Index data for fast lookup
    let data_index: Arc<LookupMap> = index_data(data.clone())?.into();
    println!("Indexing complete.");

    // Define routes
    let route_show = warp::path("show").and(warp::fs::dir(settings.NLR_STATIC_HTTP.clone()));
    let route_lines = routes::lines(data.clone(), data_index.clone());
    let route_points = routes::points(data.clone(), data_index.clone());
    let route_lines_batch = routes::lines_batch(data.clone(), data_index.clone());

    // Build routes
    let filter = route_show.or(
        route_lines
        .or(route_points)
        .or(
            route_lines_batch
            .with(warp::compression::gzip())
        )
        .with(wrap_fn(echo_x_request_id))
    );

    // Serve
    let address:SocketAddr = (settings.NLR_ADDR, settings.NLR_PORT).into();
    println!("Serving at {:?}", address);
    warp::serve(filter).run(address).await;
    Ok(())
}
