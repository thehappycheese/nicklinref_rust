use std::{sync::Arc, net::SocketAddr, error::Error};
use clap::Parser;
use warp::{Filter, wrap_fn};

mod helpers;
use helpers::{
    echo_x_request_id
};

mod routes;

mod data;
use data::{
    IndexedData,
};

mod settings;
use settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Read settings from command line of environment variables
    // see run with --help to see nice instructions generated by clap
    let settings:Settings = Settings::parse();

    // Load data
    let indexed_data:Arc<_> = IndexedData::load(
        &settings.NLR_DATA_FILE,
        &settings.NLR_DATA_SOURCE_URL,
        &settings.NLR_FORCE_UPDATE_DATA
    ).await?.into();

    // Define routes
    let route_static_folder = warp::fs::dir(settings.NLR_STATIC_HTTP.clone());
    let route_show = warp::path("show").and(route_static_folder);
    let route_lines = routes::lines(indexed_data.clone());
    let route_points = routes::points(indexed_data.clone());
    let route_lines_batch = routes::lines_batch(indexed_data.clone());

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
