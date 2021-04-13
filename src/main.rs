mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;
mod decode_query_parameters;

use std::sync::Arc;
use std::convert::Infallible;

use warp::Filter;

use config_loader::Settings;
use update_data::{update_data, load_data, perform_analysis, LookupMap};
use decode_query_parameters::{QueryParameters};
use esri_serde::{LayerSaved, LayerSavedFeature};



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
		.and_then(|full_path:warp::path::FullPath, q:QueryParameters, data:Arc<LayerSaved>, data_map:Arc<LookupMap>| async move{

			let feature_indexes = match q.road.chars().next(){
				Some(fl)=>{
					match data_map.get(&fl) {
						Some(mp1) => mp1.get(&q.road),
						None=>None
					}
				},
				None=>None
			};
			if let Some(feature_indexes) = feature_indexes{
				println!("lookup of {} from features[{} to {}]",q.road,feature_indexes.0,feature_indexes.1);
				let features = &data.features[feature_indexes.0..feature_indexes.1];
				let applicable_features:Vec<&LayerSavedFeature> = features.iter().filter(|&item| {
					item.attributes.END_SLK>q.slk_from && item.attributes.START_SLK<q.slk_to
				}).collect();
				Ok(format!("{:?} >> {:?} >> num features in road: {}\n\n{:?}", full_path, q, applicable_features.len(), applicable_features))
			}else{
				Err(warp::reject())
			}
		});
	let address = ([0,0,0,0],8080);
	println!("about to serve at  {:?}", address);
	warp::serve(route1).run(address).await;
	
	Ok(())
}
