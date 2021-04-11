use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use lz_fear;
use reqwest;
use serde_json;

use crate::basic_error::BasicError;
use crate::config_loader::Settings;
use crate::esri_serde::{Cwy, LayerDownloadChunk, LayerSaved, LayerSavedFeature};


pub async fn update_data(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	if let Err(e) = fs::remove_dir_all(&s.data_dir) {
		println!("Tried to delete data folder and contents but {}", e)
	}

	fs::create_dir_all(&s.data_dir)?;
	let file_out = File::create(Path::new(&s.data_dir).join(Path::new("output.json.lz4")))?;

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
		document_to_save.features.extend(
			json.features
				.iter()
				.map(|item| LayerSavedFeature::from(item)),
		);

		if !json.exceededTransferLimit {
			break;
		}
	}

	document_to_save
		.features
		.sort_by(|a, b| a.attributes.cmp(&b.attributes));
	let _map = perform_analysis(&document_to_save);

	let res = serde_json::to_vec(&document_to_save)?;
	let compressor = lz_fear::framed::CompressionSettings::default();
	compressor.compress(&res[..], &file_out)?;

	Ok(document_to_save)
}

pub async fn load_data<'a>(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	let file_in_json = File::open(Path::new(&s.data_dir).join(Path::new("output.json.lz4")))?;
	let decomp = lz_fear::framed::decompress_frame(file_in_json)?;
	let result: LayerSaved = serde_json::from_reader(&mut &decomp[..])?;
	
	Ok(result)
}


type LookupMap<'a> = HashMap<char, HashMap<String, HashMap<Cwy, & 'a [LayerSavedFeature]>>>;

pub fn perform_analysis<'b>(
	layer: & 'b LayerSaved,
) -> Result<LookupMap<'b>, Box<dyn std::error::Error>> {
	let mut result:LookupMap<'b> = HashMap::new(); // map_from_first_letter_to_roads

	
	let (mut previous_road, mut previous_cwy) = match layer.features.iter().next(){
		Some(first_feature)=>(&first_feature.attributes.ROAD, &first_feature.attributes.CWY),
		None=>{return Err(Box::new(BasicError::new("No features were recieved")))}
	};

	let mut current_slice_start = 0; // inclusive of that index
	
	let mut i: usize = 1;
	while i < layer.features.len() {
		let feature = &layer.features[i];

		if previous_road != &feature.attributes.ROAD || previous_cwy != &feature.attributes.CWY {
			
			
			let first_letter = match feature.attributes.ROAD.chars().next() {
				Some(fl) => fl,
				None => ' ',
			};

			let map_from_road_to_cwy = result.entry(first_letter).or_default();

			let map_from_cwy_to_slice = map_from_road_to_cwy
				.entry(previous_road.clone())
				.or_default();
			
			// the i'th item does not have the same ROAD and CWY as the (i-1)'th  item.
			// let current_slice_end = i; // exclusive
			map_from_cwy_to_slice
				.entry(feature.attributes.CWY.clone())
				.or_insert(&layer.features[current_slice_start..i]);
			//println!("slice from feature [{}..{}] for road {} and {:?}",current_slice_start,i,previous_road,previous_cwy);

			current_slice_start = i; // inclusive
		}
		
		previous_road = &feature.attributes.ROAD;
		previous_cwy = &feature.attributes.CWY;
		i += 1;
	}

	// println!(">>>Analysis Completed");
	// println!("Result top level has {} keys", result.keys().len());
	// for key in result.keys(){
	// 	println!("Key: '{}' has {} sub-keys", key, result[key].keys().len());
	// }


	Ok(result)
}
