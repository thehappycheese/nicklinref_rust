use std::fs;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use lz_fear;
use reqwest;
use serde_json;

use crate::helpers::ErrorWithMessage;
use crate::settings::Settings;
use super::esri_serde::{LayerDownloadChunk, LayerSaved, LayerSavedFeature};

pub async fn update_data_from_service(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {

	if let Err(e) = fs::remove_file(&s.NLR_DATA_FILE) {
		println!("Tried to delete the data file '{}' but: {}", &s.NLR_DATA_FILE, e);
		println!("Will attempt to proceed assuming the file does not exist");
	}

	let file_out = match File::create(Path::new(&s.NLR_DATA_FILE)){
		Ok(file)=>file,
		Err(e)=>{
			println!("Fatal: Tried to create a data file '{}' but: {}", &s.NLR_DATA_FILE, e);
			return Err(Box::new(e));
		}
	};

	let mut document_to_save = LayerSaved {
		features: Vec::with_capacity(181000), // 180303
	};

	let mut offset: usize = 0;
	println!("Downloading fresh data");
	loop {
		print!(".");
		let url = format!("{}&resultOffset={}", s.NLR_DATA_SOURCE_URL.clone(), offset);
		let json: LayerDownloadChunk = reqwest::get(url).await?.json().await?;
		if json.geometryType != "esriGeometryPolyline" {
			return Err(Box::new(ErrorWithMessage::new("Rest service returned an object that did not have geometryType:esriGeometryPolyline")));
		}
		offset += json.features.len();
		document_to_save.features.extend(
			json.features
				.iter()
				.map(|item| LayerSavedFeature::from(item)),
		);

		if !json.exceededTransferLimit {
			break;
		}
	}

	println!("Download completed. Sorting data.");
	document_to_save
		.features
		.sort_by(|a, b| a.attributes.cmp(&b.attributes));

	println!("Saving data");
	let res = serde_json::to_vec(&document_to_save)?;
	let compressor = lz_fear::framed::CompressionSettings::default();
	compressor.compress(&res[..], &file_out)?;

	Ok(document_to_save)
}

pub fn load_data_from_file(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	println!("Loading data from file.");
	let file_in_json = File::open(Path::new(&s.NLR_DATA_FILE))?;
	let decomp = lz_fear::framed::decompress_frame(file_in_json)?;
	let result: LayerSaved = serde_json::from_reader(&mut &decomp[..])?;
	Ok(result)
}

