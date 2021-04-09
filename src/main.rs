mod config_loader;
mod update_data;
mod esri_serde;
mod basic_error;

use config_loader::Settings;
use update_data::{update_data, load_data};
use std::sync::Arc;
use std::{time};
use tokio::time::{sleep, Duration};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let instant_start_main = time::Instant::now();
	println!("main started");
	let s:Arc<Settings> = Settings::new().expect("le fuckky config.").into();
	let dat = match load_data(&s).await {
		Ok(res) => res,
		Err(e) => {
			println!(
				"{:?}  - Failed to load from cache due to error {}. Now attempting to re-download.",
				time::Instant::now().duration_since(instant_start_main),
				e,
			);
			update_data(&s).await?
		}
	};
	println!(
		"{:?}  - loaded {} features and ready to perform analysis then start server. will wait 40 seconds for RAM check.",
		time::Instant::now().duration_since(instant_start_main),
		dat.features.len()	
	);
	sleep(Duration::new(40, 0)).await;
	println!(
		"{:?}  - DONE",
		time::Instant::now().duration_since(instant_start_main),
	);

	

	Ok(())
}
