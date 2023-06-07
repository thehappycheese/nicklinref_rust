use std::{path::Path, fs::File, error::Error};

use lz_fear;
use reqwest;
use serde_json;

use super::esri_serde::{LayerDownloadChunk, LayerSaved, LayerSavedFeature};


pub async fn read_or_update_cache_data(
    path_to_data_cache_file:&Path,
    url_to_download_new_data:&String,
    force_update:&bool,
) -> Result<LayerSaved, Box<dyn Error>> {
    
    if !force_update {
        match load_data_from_file(path_to_data_cache_file) {
            Ok(response) => return Ok(response),
            Err(error_message) => println!("Error while loading data: {}", error_message),
        }
    }

    if let Some(parent) = path_to_data_cache_file.parent() {
        if !parent.is_dir() {
            std::fs::create_dir_all(parent)?;
        }
    }

    if path_to_data_cache_file.is_file() {
        std::fs::remove_file(path_to_data_cache_file)?;
    }
    
    println!("Downloading fresh data........");
    let new_data = download_data(url_to_download_new_data).await?;
    
    println!("Saving data");
    let file_out = File::create(path_to_data_cache_file)?;
    let res = serde_json::to_vec(&new_data)?;
    let compressor = lz_fear::framed::CompressionSettings::default();
    compressor.compress(&res[..], &file_out)?;

    Ok(new_data)
    
}


pub async fn download_data(
        url:&String,
    ) -> Result<LayerSaved, Box<dyn std::error::Error>> {
    
	let mut document_to_save = LayerSaved {
		features: Vec::with_capacity(183_000),
	};

	let mut offset: usize = 0;
	
	loop {
		print!(".");
		let url = format!("{}&resultOffset={}", url, offset);
		let json: LayerDownloadChunk = reqwest::get(url).await?.json().await?;
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
    Ok(document_to_save)
}

pub fn load_data_from_file(s:&Path) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	println!("Loading data from file.");
    let file = File::open(s)?;
	let lz_frame_reader = lz_fear::framed::LZ4FrameReader::new(file)?;
    let lz_frame_io_reader = lz_frame_reader.into_read();
    let result: LayerSaved = serde_json::from_reader(lz_frame_io_reader)?;
	Ok(result)
}

