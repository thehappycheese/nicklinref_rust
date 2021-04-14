mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;
mod decode_query_parameters;

use std::sync::Arc;
use std::convert::Infallible;

use warp::Filter;
use nickslinetoolsrust::vector2::Vector2;
use nickslinetoolsrust::linestring::{LineString, LineStringy, LineStringMeasured};

use config_loader::Settings;
use update_data::{update_data, load_data, perform_analysis, LookupMap, RoadDataByCwy};
use decode_query_parameters::{QueryParameters};
use esri_serde::{LayerSaved, LayerSavedFeature, Cwy};


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

	let route1 = warp::any()
		.and(warp::path::full())
		.and(warp::query())
		.and(clone_arc(data.clone()))
		.and(clone_arc(data_map.clone()))
		.and_then(|full_path:warp::path::FullPath, query:QueryParameters, data:Arc<LayerSaved>, data_map:Arc<LookupMap>| async move{

			let road_data:&RoadDataByCwy = match match query.road.chars().next(){
				Some(first_letter)=>{
					match data_map.get(&first_letter) {
						Some(mp1) => mp1.get(&query.road),
						None=>None
					}
				},
				None=>None
			}{
				Some(data_lookup_subtable)=>data_lookup_subtable,
				None=>{return Err(warp::reject())}
			};
			
			#[derive(Debug)]
			struct ls_dbg{
				slk_from:f32,
				slk_to:f32,
				ls:LineString,
				cwy:Cwy
			}

			let features:Vec<ls_dbg> = query.cwy
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
						let len_requested = query.slk_to-query.slk_from;
						let item_len_km = item.attributes.END_SLK - item.attributes.START_SLK;
						let frac_start = (query.slk_from-item.attributes.START_SLK)/item_len_km;
						let frac_end = (query.slk_to-item.attributes.START_SLK)/item_len_km;
						if let (_, Some(b), _) = lsm.double_cut_linestring(frac_start.into(), frac_end.into()) {
							Some(ls_dbg{
								slk_from:item.attributes.START_SLK,
								slk_to:item.attributes.END_SLK,
								ls:b.to_line_string(),
								cwy:item.attributes.CWY.clone()
							})
						}else{
							None
						}
					}else{
						None
					}
				})
				.collect();


			Ok(format!("{:?} >> {:?} >> num features in road: {}\n\n{:?}", full_path, query, features.len(), features))

		});
	let address = ([0,0,0,0],8080);
	println!("about to serve at  {:?}", address);
	warp::serve(route1).run(address).await;
	
	Ok(())
}
