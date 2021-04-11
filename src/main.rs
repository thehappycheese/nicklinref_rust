mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;
mod decode_query_parameters;

use config_loader::Settings;
use update_data::{update_data, load_data, perform_analysis};
use std::sync::Arc;
use decode_query_parameters::{QueryParameters};
use esri_serde::{LayerSaved};
use warp::Filter;
use std::future;




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
	let data_map = perform_analysis(&data)?;

	println!("loaded {} features and ready to perform analysis then start server.", data.features.len());

	

	let route1 = warp::any()
		.and(warp::path::full())
		.and(warp::query())
		.and_then(|full_path:warp::path::FullPath, q:QueryParameters| async {
			let features = match q.road.chars().next(){
				Some(fl)=>{
					match data_map.get(&fl) {
						Some(mp1) => mp1.get(&q.road),
						None=>None
					}
				},
				None=>None
			};
			if let Some(features) = features{
				let num_satis = features.iter().filter(|&item|item.attributes.END_SLK>=q.slk_from || item.attributes.START_SLK<=q.slk_to).count();
				Ok(format!("{:?} >> {:?} >> num features in road: {}", full_path, q, num_satis))
			}else{
				Err(warp::reject())
			}
		});
	let address = ([0,0,0,0],8080);
	println!("about to serve at  {:?}", address);
	warp::serve(route1).run(address).await;
	
	Ok(())
}
