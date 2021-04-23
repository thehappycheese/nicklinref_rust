mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;
mod decode_query_parameters;
mod unit_conversion;

use std::str;
use std::sync::Arc;
use std::convert::Infallible;

use warp::Filter;
use warp::http::{StatusCode};
use bytes;
use config_loader::Settings;

use nickslinetoolsrust::linestring::{LineStringy, LineStringMeasured};
use unit_conversion::convert_metres_to_degrees;
use update_data::{update_data, load_data, perform_analysis, LookupMap, RoadDataByCwy};
use decode_query_parameters::{QueryParameters, OutputFormat, QueryParameterBatch};
use esri_serde::{LayerSaved};
use std::net::{IpAddr, SocketAddr};
use basic_error::BasicErrorWarp;




fn clone_arc<T>(something:T) -> impl warp::Filter<Extract=(T,), Error=Infallible> + Clone
where T:Send+Sync+Clone{
	warp::any().map(move || something.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	
	let s:Arc<Settings> = Settings::new()?.into();

	let data:Arc<LayerSaved> = match load_data(&s).await {
		Ok(res) => res,
		Err(e) => {
			// TODO: add user input confirmation?
			println!("Failed to load from cache due to error {}. Will try re-download.", e);
			update_data(&s).await?
		}
	}.into();

	println!("loaded {} features and ready to perform analysis then start server.", data.features.len());

	let data_map:Arc<LookupMap> =  perform_analysis(data.clone())?.into();

	let route_query = warp::path("query")
		.and(warp::query())
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_map.clone()))
		.and_then(|query:QueryParameters, data:Arc<LayerSaved>, data_map:Arc<LookupMap>| async move{

			let road_data:&RoadDataByCwy = match match query.road.chars().next(){
				Some(first_letter)=>{
					match data_map.get(&first_letter) {
						Some(mp1) => mp1.get(&query.road),
						None=>{return Err(warp::reject::custom(BasicErrorWarp::new("first letter not found. lookup failed")))}
					}
				},
				None=>{return Err(warp::reject::custom(BasicErrorWarp::new("could not get first letter of road")))}
			}{
				Some(data_lookup_subtable)=>data_lookup_subtable,
				None=>{return Err(warp::reject::custom(BasicErrorWarp::new("full road name not found. lookup failed")))}
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
						let frac_start = (query.slk_from-item.attributes.START_SLK)/item_len_km;
						let frac_end = (query.slk_to-item.attributes.START_SLK)/item_len_km;

						match lsm.cut_twice(frac_start.into(), frac_end.into()){
							(_, Some(b), _) => if query.offset == 0.0 {
										Some(b.to_line_string())
									}else{
										let degree_offset:f64 = convert_metres_to_degrees(query.offset.into());
										b.offset_basic(degree_offset)
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

		});
	
	
	let route_batch = warp::post()
		.and(warp::path("batch"))
		.and(warp::body::bytes())
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_map.clone()))
		.and_then(|body:bytes::Bytes, _data:Arc<LayerSaved>, _data_map:Arc<LookupMap>| async move{
			let m:QueryParameterBatch = body.into();
			Ok(format!("{}", b))
		});
		// }).recover(|error:warp::reject::Rejection|{
		// 	// TODO: blames the user for all errors:
			
		// 	if false{
		// 	 	Err(error) // this nonsense makes the type checker happy?
		// 	 }else{
		// 		Ok(warp::reply::with_status(warp::reply(), StatusCode::BAD_REQUEST))	
		// 	}
		// });



	let route_static = warp::path("show").and(warp::fs::dir(s.static_dir.clone()));
	let address:SocketAddr = SocketAddr::new(IpAddr::V4(s.server), s.port);
	println!("about to serve at  {:?}", address);
	warp::serve(
		route_static
		.or(route_query)
		.or(route_batch)
	).run(address).await;
	
	Ok(())
}
