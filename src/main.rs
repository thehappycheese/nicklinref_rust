mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;
mod decode_query_parameters;
mod unit_conversion;

use std::{convert::TryFrom, net::IpAddr, str};
use std::sync::Arc;
use std::convert::Infallible;

use warp::Filter;
use bytes;
use config_loader::Settings;

use nickslinetoolsrust::linestring::{LineStringy, LineStringMeasured};
use unit_conversion::convert_metres_to_degrees;
use update_data::{update_data, load_data, perform_analysis, LookupMap, RoadDataByCwy};
use decode_query_parameters::{QueryParameters, OutputFormat, QueryParameterBatch};
use esri_serde::{LayerSaved};
use std::net::{SocketAddr};
use basic_error::BasicErrorWarp;



/// Moves a clone of an Arc<T> into a warp filter chain.
/// The closure here takes ownership of the first clone, and provides yet another clone of the arc whenever it is called.
/// I think this lets the first Arc clone live as long as the filter
/// but I spent HOURS trying to move a reference to data and data_index
/// into the filter closures with no success. This is the only way that works,
/// I can only assume this is idiomatic rust. Idiotic more like.
fn clone_arc<T>(something:T) -> impl warp::Filter<Extract=(T,), Error=Infallible> + Clone
where T:Send+Sync+Clone{
	warp::any().map(move || something.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	
	let settings:Arc<Settings> = match Settings::new(){
		Ok(settings)=>settings,
		Err(e)=>panic!("Unable to load config.json:  {}", e)
	}.into();


	let data:Arc<LayerSaved> = match load_data(&settings).await {
		Ok(res) => res,
		Err(e) => {
			// TODO: add user input confirmation?
			println!("Failed to load from cache due to error {}. Will try re-download.", e);
			update_data(&settings).await?
		}
	}.into();


	println!("Loaded {} features.", data.features.len());


	let data_index:Arc<LookupMap> =  perform_analysis(data.clone())?.into();
	
	
	println!("Indexing complete.");


	let route_query = 
		warp::get()
		.and(warp::path("query"))
		.and(warp::query())
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_index.clone()))
		.and_then(|query:QueryParameters, data:Arc<LayerSaved>, data_index:Arc<LookupMap>| async move{
			match get_stuff(&query,&data,&data_index){
				Ok(s)=>Ok(s),
				Err(e)=>Err(warp::reject::custom(BasicErrorWarp::new(e)))
			}
		});
	
	
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
				.map(|query| match get_stuff(query, &data, &data_index){
					Ok(x)=>x,
					Err(_)=>"null".to_string()
				})
				.collect::<Vec<String>>()
				.join(",");
			if false{
				return Err(warp::reject::custom(BasicErrorWarp::new("to make the typechecker happy")))
			}
			Ok("[".to_string() + &f + "]")
		});

	
	let route_show = 
		warp::path("show")
		.and(warp::fs::dir(settings.static_dir.clone()));

	// TODO: warp docs recommend that all rejections should be handled. I haven't figured that out just yet.

	let route_health = warp::get().and(warp::path("health")).and_then(health_handler);
	
	let filter = 
		route_show
		.or(route_query)
		.or(route_batch)
		.or(route_health)
		.with(
			warp::cors()
			.allow_any_origin() 
		);
		// TODO: we can probably limit this to the PowerBI visual, rather than allow_any_origin.
		//  I don't know how PowerBI desktop would like that...?

	let address:SocketAddr = SocketAddr::new(IpAddr::V4(settings.server), settings.port);
	println!("Serving at {:?}", address);
	warp::serve(
		filter
	)
	.tls()
	.cert_path(&settings.cert_path)
	.key_path(&settings.key_path)
	.run(address).await;
	
	Ok(())
}

async fn health_handler() -> Result<impl warp::Reply, Infallible> {
	Ok("OK")
}



fn get_stuff(query:&QueryParameters, data:&Arc<LayerSaved>, data_index:&Arc<LookupMap>)->Result<String, & 'static str>{
	let road_data:&RoadDataByCwy = match match query.road.chars().next(){
		Some(first_letter)=>{
			match data_index.get(&first_letter) {
				Some(mp1) => mp1.get(&query.road),
				None=>{return Err("road lookup failed, first letter did not match any lookup tables.")}
			}
		},
		None=>{return Err("could not get first letter of road")}
	}{
		Some(data_lookup_sub_table)=>data_lookup_sub_table,
		None=>{return Err("full road name not found. lookup failed")}
	};

	let features = query.cwy
		.into_iter()
		.filter_map(|cwy|{
			if let Some(indexes) = road_data[&cwy]{
				Some(&data.features[indexes.0..indexes.1])
			}else{
				None
			}
		})
		.flatten()
		.filter_map(|item|{
			if item.attributes.END_SLK>query.slk_from && item.attributes.START_SLK<query.slk_to{

				let lsm:LineStringMeasured = LineStringMeasured::from_vec(&item.geometry);
				
				let item_len_km = item.attributes.END_SLK - item.attributes.START_SLK;
				let frac_start = (query.slk_from-item.attributes.START_SLK) / item_len_km;
				let frac_end = (query.slk_to-item.attributes.START_SLK) / item_len_km;

				match lsm.cut_twice(frac_start.into(), frac_end.into()){
					(_, Some(b), _) => if query.offset == 0.0 {
								Some(b.to_line_string())
							}else{
								let degree_offset:f64 = convert_metres_to_degrees(query.offset.into());
								b.offset_basic(-degree_offset)
							},
					_=>None
				}

			}else{
				None
			}
		});

		match query.f{
			OutputFormat::JSON => {
				let line_string_string = features
					.map(|linestring|{
							"[".to_string() + &linestring.points.iter().filter_map(|vertex| serde_json::to_string(vertex).ok()).collect::<Vec<String>>().join(",") + "]"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok("[".to_string() + &line_string_string + "]")
			},
			OutputFormat::GEOJSON => {
				let line_string_string = features
					.map(|linestring|{
							"[".to_string() + &linestring.points.iter().filter_map(|vertex| serde_json::to_string(vertex).ok()).collect::<Vec<String>>().join(",") + "]"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok( r#"{"type":"Feature", "geometry":{"type":"MultiLineString", "coordinates":["#.to_string() + &line_string_string + "]}}")
			},
			OutputFormat::WKT => {
				let line_string_string = features
					.map(|linestring|{
							"(".to_string() + &linestring.points.iter().map(|vertex| format!("{} {}", vertex.x, vertex.y)).collect::<Vec<String>>().join(",") + ")"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok("MULTILINESTRING (".to_string() + &line_string_string + ")")
			}
		}
}