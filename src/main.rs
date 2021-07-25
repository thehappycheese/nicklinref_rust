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
use geoprocessing::{get_linestring, get_points};
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
		//.and(warp::path::end())
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_index.clone()));

	let no_path_lines = 
		no_path.clone()
		.and(warp::query())
		.and_then(|data:Arc<LayerSaved>, data_index:Arc<LookupMap>, query:QueryParametersLine| async move{
			match get_linestring(&query, &data, &data_index){
				Ok(s)=>Ok(s),
				Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
			}
		});

	let no_path_points = 
		no_path
		.and(warp::query())
		.and_then(|data:Arc<LayerSaved>, data_index:Arc<LookupMap>, query:QueryParametersPoint| async move{
			match get_points(&query, &data, &data_index){
				Ok(s)=>Ok(s),
				Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
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
		.and_then(|body:bytes::Bytes, data:Arc<LayerSaved>, data_index:Arc<LookupMap>| async move{
			
			
			let batch_query = QueryParameterBatch::try_from(body).or(Err(warp::reject::custom(BasicErrorWarp::new("Unable to parse query parameters"))))?;

			// TODO: could add some intelligence here... repeated lookups in the hash map can be avoided when multiple queries have the same road number and cwy.
			let f = batch_query.0
				.iter()
				.map(|query| match get_linestring(query, &data, &data_index){
					Ok(x)=>x,
					Err(_)=>"null".to_string()
				})
				.collect::<Vec<String>>()
				.join(",");
			
			if false{
			 	return Err(warp::reject::custom(BasicErrorWarp::new("to make the typechecker happy")))
			}

			Ok(
				"[".to_string() + &f + "]"
			)
		});

	
	
	let filter = 
		route_show
		.or(no_path_lines)
		.or(no_path_points)
		.or(route_batch)
		.with(
			warp::cors()
			.allow_any_origin() 
		);

	
	let address = settings.get_socket_address();
	println!("Serving at {:?}", address);
	warp::serve(filter).run(address).await;
	
	Ok(())
}



