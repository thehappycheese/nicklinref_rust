use std::fs;
use std::fs::File;
use std::path::Path;

use crate::basic_error::BasicError;
use crate::config_loader::Settings;
use crate::esri_serde::{LayerDownloadChunk, LayerSaved, LayerSavedFeature};
use std::sync::Arc;

use bson;
use lz_fear;
use reqwest;

use std::{time};


pub async fn update_data(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	println!("update_data started");
	let instant_start_update_data = time::Instant::now();
	if let Err(e) = fs::remove_dir_all(&s.data_dir) {
		println!("Tried to delete data folder and contents but {}", e)
	}
	fs::create_dir_all(&s.data_dir)?;
	let file_out_bson = File::create(Path::new(&s.data_dir).join(Path::new("output.bson.lz4")))?;
	let mut document_to_save = LayerSaved {
		features: Vec::with_capacity(180246),
	};
	let mut offset: usize = 0;
	loop {
		let url = format!("{}&resultOffset={}", s.data_url.clone(), offset);
		let json: LayerDownloadChunk = reqwest::get(url).await?.json().await?;
		if json.geometryType != "esriGeometryPolyline" {
			return Err(Box::new(BasicError::new("Rest service returned an object that did not have geometryType:esriGeometryPolyline")));
		}
		offset += json.features.len();
		document_to_save.features.extend(json.features.iter().map(|item| LayerSavedFeature::from(item)));

		if json.exceededTransferLimit {
			println!(
				"{:?}  - Downloaded a chunk at resultOffset:{} - features.len() == {}",
				time::Instant::now().duration_since(instant_start_update_data),
				offset,
				document_to_save.features.len()
			);
		} else {
			println!(
				"{:?}  - FINISHED at at resultOffset:{} - features.len() == {}",
				time::Instant::now().duration_since(instant_start_update_data),
				offset,
				document_to_save.features.len()
			);
			break;
		}
	}

	let res2 = bson::to_document(&document_to_save).unwrap();

	let compressor = lz_fear::framed::CompressionSettings::default();

	let mut binary_bson = vec![];
	res2.to_writer(&mut binary_bson)?;

	compressor.compress(&binary_bson[..], &file_out_bson)?;

	Ok(document_to_save)
}

pub async fn load_data(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	let instant_start_load_data = time::Instant::now();
	println!("0.00ms - load_data started");
	let file_in_bson = File::open(Path::new(&s.data_dir).join(Path::new("output.bson.lz4")))?;
	println!("{:?}  - trying to decompress data",time::Instant::now().duration_since(instant_start_load_data));
	let decomp = lz_fear::framed::decompress_frame(file_in_bson)?;
	println!("{:?}  - trying to create document from data", time::Instant::now().duration_since(instant_start_load_data));
	let result:LayerSaved = bson::from_document(bson::Document::from_reader(&mut &decomp[..])?)?;
	println!("{:?}  - load_data ended", time::Instant::now().duration_since(instant_start_load_data));
	Ok(result)
}
