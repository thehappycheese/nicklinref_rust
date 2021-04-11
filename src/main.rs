mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;
mod decode_query_parameters;

use config_loader::Settings;
use update_data::{update_data, load_data, perform_analysis};
use std::sync::Arc;
use decode_query_parameters::{QueryParameters};

use warp::Filter;





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	
	let s:Arc<Settings> = Settings::new()?.into();

	let data = match load_data(&s).await {
		Ok(res) => res,
		Err(e) => {
			// TODO: add user input confirmation?
			println!("Failed to load from cache due to error {}. Will try re-download.", e);
			update_data(&s).await?
		}
	};
	let data_map = perform_analysis(&data)?;

	println!("loaded {} features and ready to perform analysis then start server.", data.features.len());

	

	let route1 = warp::any()
		.and(warp::path::full())
		.and(warp::query())
		.map(|full_path:warp::path::FullPath, q:QueryParameters|{
			let features = match q.road.chars().next(){
				Some(fl)=>{
					match data_map.get(&fl){
						Some(mp1)=> match mp1.get(&q.road){
								Some(mp2)=>{
									let res = Vec::new();
									Some(res)
								},
								None=>None
							},
						None=>None
					}
				},
				None=>None
			};
			Ok(format!("huh {:?} {:?}", full_path, q))
		});
	let address = ([0,0,0,0],8080);
	println!("about to serve at  {:?}", address);
	warp::serve(route1).run(address).await;
	
	Ok(())
}
