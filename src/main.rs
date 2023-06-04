use std::convert::TryFrom;
use std::sync::Arc;

//use bytes;
use warp::{Filter, wrap_fn};

mod helpers;
use helpers::{
	echo_x_request_id,
	with_shared_data,
	ErrorWithMessage
};

mod esri_serde;
use esri_serde::LayerSaved;

mod geoprocessing;
use geoprocessing::{
	get_linestring,
	get_linestring_m,
	get_points
};

mod query_parameters;
use query_parameters::{
	QueryParameterBatch,
	QueryParametersLine,
	QueryParametersPoint
};

mod settings;
use settings::Settings;

mod update_data;
use update_data::{load_data, perform_analysis, update_data, LookupMap};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read settings
    let settings:Arc<Settings> = match Settings::get(){
        Ok(settings)=>settings,
        Err(e)=>{
            println!("Unable to load configuration from environment variables or from any .json file specified with the --config command line option:  {}", e);
            return Err(e);
        }
    }.into();

    // load data
    let data: Arc<LayerSaved> = match load_data(&settings) {
        Ok(res) => res,
        Err(e) => {
            println!(
                "Failed to load from cache due to error {}. Will try re-download.",
                e
            );
            update_data(&settings).await?
        }
    }.into();
    println!("Loaded {} features.", data.features.len());

    // index data for fast lookup
    let data_index: Arc<LookupMap> = perform_analysis(data.clone())?.into();
    println!("Indexing complete.");

    // Serve static HTML/js directory
    let route_show = warp::path("show").and(warp::fs::dir(settings.NLR_STATIC_HTTP.clone()));

    // generic query base
    // ignores path
    // note: previous versions included `.and(warp::path::end())`
    //       but this was removed as it conflicted with the
    //       static file route for obscure and frustrating reasons
    let route_get_query = 
        warp::get()
        .and(with_shared_data(data.clone()))
        .and(with_shared_data(data_index.clone()));

    // Line geometry is requested
    let route_lines_query = 
        route_get_query.clone()
        .and(warp::query())
        .and_then(|
                data: Arc<LayerSaved>,
                data_index: Arc<LookupMap>,
                query: QueryParametersLine
            | async move {
                if query.m {
                    match get_linestring_m(&query, &data, &data_index) {
                        Ok(s) => Ok(s),
                        Err(e) => Err(ErrorWithMessage::reject(e)),

                    }
                } else {
                    match get_linestring(&query, &data, &data_index) {
                        Ok(s) => Ok(s),
                        Err(e) => Err(ErrorWithMessage::reject(e)),
                    }
                }
            });

    // Point geometry is requested
    let route_points_query = 
        route_get_query.clone()
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
            });

    // generic POST query base
    // requires path /batch,
    // extracts request body as bytes
    let route_post_query = warp::post()
        .and(warp::path("batch").and(warp::path::end()))
        .and(with_shared_data(data.clone()))
        .and(with_shared_data(data_index.clone()))
        .and(warp::body::bytes());

    // Batch Line Geometry is requested
    let route_lines_batch_query =
        route_post_query.clone()
        .and_then(|
                data: Arc<LayerSaved>,
                data_index: Arc<LookupMap>,
                body: bytes::Bytes
            | async move {
                if let Ok(batch_query) = QueryParameterBatch::try_from(body) {
                    let result_string = batch_query
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
                    Err(ErrorWithMessage::reject("Unable to parse batch query parameters"))
                }
            });

    let filter = route_show.or(
        route_lines_query
        .or(route_points_query)
        .or(
            route_lines_batch_query
            .with(warp::compression::gzip())
        )
        .with(wrap_fn(echo_x_request_id))
    );

    let address = settings.get_socket_address();
    println!("Serving at {:?}", address);
    warp::serve(filter).run(address).await;
    Ok(())
}
