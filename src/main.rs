mod settings;
mod update_data;
mod esri_serde;
mod basic_error;
mod query_parameters;
mod unit_conversion;
mod geoprocessing;
//mod echo_header;


use std::{convert::TryFrom};
use std::sync::Arc;
use std::convert::Infallible;

use warp::http::response::Builder;
use warp::{Filter, http::Response};
use bytes;
use settings::Settings;


use update_data::{update_data, load_data, perform_analysis, LookupMap};
use esri_serde::{LayerSaved};
use query_parameters::{ QueryParameterBatch};
use geoprocessing::{get_linestring, get_points, get_linestring_m};
//use echo_header::{echo_header_x_request_id};


use crate::query_parameters::{QueryParametersLine, QueryParametersPoint};

/// Moves a clone of an Arc<T> into a warp filter chain.
/// The closure here takes ownership of the first clone, 
/// and provides yet another clone of the arc whenever it is called.
/// I think this lets the first Arc clone live as long as the filter
/// I spent HOURS trying to move a reference to data and data_index
/// directly from main into the .and_then() filter closures with no success.
/// This is the only way i have found that works, therefore I can only assume this is idiomatic warp/rust.
fn clone_arc<T>(something:T) -> impl warp::Filter<Extract=(T,), Error=Infallible> + Clone
where T:Send+Sync+Clone{
	warp::any().map(move || something.clone())
}

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
	let data:Arc<LayerSaved> = match load_data(&settings) {
		Ok(res) => res,
		Err(e) => {
			println!("Failed to load from cache due to error {}. Will try re-download.", e);
			update_data(&settings).await?
		}
	}.into();
	println!("Loaded {} features.", data.features.len());

	// index data for fast lookup
	let data_index:Arc<LookupMap> =  perform_analysis(data.clone())?.into();
	println!("Indexing complete.");

	// Serve static HTML/js directory
	let route_show = 
		warp::path("show")
		.and(warp::fs::dir(settings.NLR_STATIC_HTTP.clone()));



	// Allow users to track their requests
	let echo_x_request_id =
		warp::any()
		.and(warp::header::optional::<String>("x-request-id"))
		.map(|request_id:Option<String>| {
			let resp = Response::builder();
			if let Some(request_id) = request_id {
				if let Ok(_) = request_id.parse::<u64>() {
					resp.header("x-request-id", request_id)
				}else{
					// if not an unsigned integer then don't echo it
					resp
				}
			}else{
				resp
			}
		});


	// generic query base
	// ignores path
	// note: previous versions included `.and(warp::path::end())`
	//       but this was removed as it conflicted with the
	//       static file route for obscure and frustrating reasons
	let route_get_query = 
		warp::get()
		.and(echo_x_request_id)
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_index.clone()));

	// Line geometry is requested
	let route_lines_query = 
		route_get_query.clone()
		.and(warp::query())
		.map(|response_builder:Builder, data:Arc<LayerSaved>, data_index:Arc<LookupMap>, query:QueryParametersLine| {
			if query.m {
				match get_linestring_m(&query, &data, &data_index){
					Ok(s)=>response_builder.status(200).body(s),
					Err(e)=>response_builder.status(500).body(format!("{}", e)),
				}
			}else{
				match get_linestring(&query, &data, &data_index){
					Ok(s)=>response_builder.status(200).body(s),
					Err(e)=>response_builder.status(500).body(format!("{}", e)),
				}
			}
		});

	// Point geometry is requested
	let route_points_query = 
		route_get_query.clone()
		.and(warp::query())
		.map(|response_builder:Builder, data:Arc<LayerSaved>, data_index:Arc<LookupMap>, query:QueryParametersPoint| {
			match get_points(&query, &data, &data_index){
				Ok(s) => response_builder.status(200).body(s),
				Err(e)  => response_builder.status(500).body(format!("{}", e)),
			}
		});

	// generic POST query base
	// requires path /batch,
	// extracts request body as bytes
	// TODO: probably should include `.and(warp::path::end())`
	//       to reject requests with invalid paths
	let route_post_query = 
		warp::post()
		.and(warp::path("batch"))
		.and(echo_x_request_id)
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_index.clone()))
		.and(warp::body::bytes());
	
	// Batch Line Geometry is requested
	let route_lines_batch_query = 
		route_post_query.clone()
		.map(|response_builder:Builder, data:Arc<LayerSaved>, data_index:Arc<LookupMap>, body:bytes::Bytes| {
			if let Ok(batch_query) = QueryParameterBatch::try_from(body){
				let result_string = batch_query
					.0
					.iter()
					.map(|query| match get_linestring(query, &data, &data_index){
						Ok(x)=>x,
						Err(_)=>"null".to_string()
					})
					.collect::<Vec<String>>()
					.join(",");
				response_builder.status(200).body(format!("[{}]", result_string))
			}else{
				response_builder.status(400).body("Unable to parse batch query parameters".to_owned())
			}
		});

	let filter = 
		route_show
		.or(route_lines_query)
		.or(route_points_query)
		.or(route_lines_batch_query.with(warp::compression::gzip()));
		
	
	let address = settings.get_socket_address();
	println!("Serving at {:?}", address);
	warp::serve(filter).run(address).await;
	Ok(())
}
