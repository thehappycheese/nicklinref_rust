mod settings;
mod update_data;
mod esri_serde;
mod basic_error;
mod query_parameters;
mod unit_conversion;
mod geoprocessing;

use std::{convert::TryFrom};
use std::sync::Arc;
use std::convert::Infallible;

use warp::Filter;
use bytes;
use settings::Settings;


use update_data::{update_data, load_data, perform_analysis, LookupMap};
use query_parameters::{ QueryParameterBatch};
use geoprocessing::{get_linestring, get_points, get_linestring_m};
use esri_serde::{LayerSaved};
use basic_error::BasicErrorWarp;

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

	
	let settings:Arc<Settings> = match Settings::get(){
		Ok(settings)=>settings,
		Err(e)=>{
			println!("Unable to load configuration from environment variables or from any .json file specified with the --config command line option:  {}", e);
			return Err(e);
		}
	}.into();


	let data:Arc<LayerSaved> = match load_data(&settings) {
		Ok(res) => res,
		Err(e) => {
			println!("Failed to load from cache due to error {}. Will try re-download.", e);
			update_data(&settings).await?
		}
	}.into();


	println!("Loaded {} features.", data.features.len());


	let data_index:Arc<LookupMap> =  perform_analysis(data.clone())?.into();
	
	
	println!("Indexing complete.");


	// let route_line = 
	// 	warp::get()
	// 	.and(warp::path("lines"))
	// 	.and(warp::query())
	// 	.and(clone_arc(data.clone()))
	// 	.and(clone_arc(data_index.clone()))
	// 	.and_then(|query:QueryParametersLine, data:Arc<LayerSaved>, data_index:Arc<LookupMap>| async move{
	// 		match get_linestring(&query, &data, &data_index){
	// 			Ok(s)=>Ok(s),
	// 			Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
	// 		}
	// 	});
	
	// let route_points = 
	// 	warp::get()
	// 	.and(warp::path("points"))
	// 	.and(warp::query())
	// 	.and(clone_arc(data.clone()))
	// 	.and(clone_arc(data_index.clone()))
	// 	.and_then(|query:QueryParametersPoint, data:Arc<LayerSaved>, data_index:Arc<LookupMap>| async move{
	// 		match get_points(&query, &data, &data_index){
	// 			Ok(s)=>Ok(s),
	// 			Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
	// 		}
	// 	});

	let no_path = 
		warp::get()
		//.and(warp::path::end())  // TODO: this was removed for some obscure frustrating reason that I don't remember
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_index.clone()));

	let no_path_lines = 
		no_path.clone()
		.and(warp::query())
		.and_then(|data:Arc<LayerSaved>, data_index:Arc<LookupMap>, query:QueryParametersLine| async move{
			if query.m {
				match get_linestring_m(&query, &data, &data_index){
					Ok(s)=>Ok(s),
					Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
				}
			}else{
				match get_linestring(&query, &data, &data_index){
					Ok(s)=>Ok(s),
					Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
				}
			}
		});

	let no_path_points = 
		no_path
		.and(warp::query())
		.and_then(|data:Arc<LayerSaved>, data_index:Arc<LookupMap>, query:QueryParametersPoint| async move{
			match get_points(&query, &data, &data_index){
				Ok(s) => Ok(s),
				Err(e)  => Err(warp::reject::custom(BasicErrorWarp::new(e)))
			}
		});
	
	let route_show = 
		warp::path("show")
		.and(warp::fs::dir(settings.NLR_STATIC_HTTP.clone()));

	
	let route_batch = 
		warp::post()
		.and(warp::path("batch"))
		.and(warp::body::bytes())
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_index.clone()))
		.and_then(|body:bytes::Bytes, data:Arc<LayerSaved>, data_index:Arc<LookupMap>| async move {
			QueryParameterBatch::try_from(body).map_err(|_|
				warp::reject::custom(BasicErrorWarp::new("Unable to parse batch query parameters"))
			).map(|batch_query|
				batch_query
				.0
				.iter()
				.map(|query| match get_linestring(query, &data, &data_index){
					Ok(x)=>x,
					Err(_)=>"null".to_string()
				})
				.collect::<Vec<String>>()
				.join(",")
			).map(|result_string|
				format!("[{}]", result_string)
			)
		});

	// This filter will help clients avoid processing out-of-order responses from the server
	let echo_request_id_filter = warp::header::optional::<String>("x-request-id")
		.map(|request_id: Option<String>| {
			let mut reply = warp::reply();
			if let Some(id) = request_id {
				// Attempt to parse the id as an integer.
				if id.parse::<i64>().is_ok() {
					warp::reply::with_header(reply, "x-request-id", id);
				}
			}
			reply
		});
	
	let filter = 
		route_show
		.or(no_path_lines .and(echo_request_id_filter))
		.or(no_path_points.and(echo_request_id_filter))
		.or(route_batch   .and(echo_request_id_filter));
		// .with(
		// 	warp::cors()
		// 	.allow_any_origin() 
		// );

	
	let address = settings.get_socket_address();
	println!("Serving at {:?}", address);
	warp::serve(filter).run(address).await;
	
	Ok(())
}



