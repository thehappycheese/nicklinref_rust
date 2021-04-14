use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::ops::Index;


use lz_fear;
use reqwest;
use serde_json;

use crate::basic_error::BasicError;
use crate::config_loader::Settings;
use crate::esri_serde::{LayerDownloadChunk, LayerSaved, LayerSavedFeature, Cwy};


pub async fn update_data(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	if let Err(e) = fs::remove_dir_all(&s.data_dir) {
		println!("Tried to delete data folder ({}) and contents but: {}", &s.data_dir, e);
		println!("Will attempt to proceed assuming the folder does not yet exist anyway");
	}

	match fs::create_dir_all(&s.data_dir){
		Ok(_)=>{},
		Err(e)=>{
			println!("Tried to create data folder ({}) but: {}", &s.data_dir, e);
			println!("Will attempt to proceed assuming the folder already exists");
		}
	};
	let file_out = File::create(Path::new(&s.data_dir).join(Path::new("output.json.lz4")))?; // errors out if file cant be created.

	let mut document_to_save = LayerSaved {
		features: Vec::with_capacity(180246),
	};

	let mut offset: usize = 0;
	println!("Downloading fresh data");
	loop {
		print!(".");
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

pub async fn load_data<'a>(s: &Arc<Settings>) -> Result<LayerSaved, Box<dyn std::error::Error>> {
	println!("Loading data from file");
	let file_in_json = File::open(Path::new(&s.data_dir).join(Path::new("output.json.lz4")))?;
	let decomp = lz_fear::framed::decompress_frame(file_in_json)?;
	let result: LayerSaved = serde_json::from_reader(&mut &decomp[..])?;
	
	Ok(result)
}




#[allow(non_snake_case)]
pub struct RoadDataByCwy{
	pub Left:Option<(usize, usize)>,
	pub Right:Option<(usize, usize)>,
	pub Single:Option<(usize, usize)>,
}
impl RoadDataByCwy{
	fn new(l:Option<(usize,usize)>, r:Option<(usize,usize)>, s:Option<(usize,usize)>)->Self{
		Self{Left:l,Right:r,Single:s}
	}
	fn new_from_cwy(cwy:&Cwy, range:(usize, usize))->Self{
		match cwy{
			Cwy::Left=>Self::new(Some(range), None, None),
			Cwy::Right=>Self::new(None, Some(range), None),
			Cwy::Single=>Self::new(None, None, Some(range)),
		}
	}
	fn with_updated_cwy(&self,cwy:&Cwy, range:(usize, usize))->Self{
		match cwy{
			Cwy::Left=>Self::new(Some(range), self.Right, self.Single),
			Cwy::Right=>Self::new(self.Left, Some(range), self.Single),
			Cwy::Single=>Self::new(self.Left, self.Right, Some(range)),
		}
	}
}
impl Index<&Cwy> for RoadDataByCwy{
	type Output = Option<(usize, usize)>;
	fn index(&self, index:&Cwy)->&Self::Output{
		match index{
			Cwy::Left=>&self.Left,
			Cwy::Right=>&self.Right,
			Cwy::Single=>&self.Single,
		}
	}
}


pub type LookupMap = HashMap<char, HashMap<String, RoadDataByCwy>>;

pub fn perform_analysis(
	layer: Arc<LayerSaved>,
) -> Result<LookupMap, Box<dyn std::error::Error>> {
	
	println!("Analysing data");
	let mut map_from_first_letter:LookupMap = HashMap::new(); // map_from_first_letter_to_roads

	
	let (
		mut previous_road,
		mut previous_cwy,
		mut first_letter,
	) = match layer.features.iter().next(){
		Some(first_feature)=>(
			&first_feature.attributes.ROAD,
			&first_feature.attributes.CWY,
			match layer.features[0].attributes.ROAD.chars().next() {
				Some(fl) => fl,
				None => ' ',
			}
		),
		None=>{return Err(Box::new(BasicError::new("Zero features recieved by perform_analysis()")))}
	};

	let mut current_slice_start = 0; // inclusive of that index
	
	let mut map_from_road_number = map_from_first_letter.entry(first_letter).or_default();
	let mut i: usize = 1;
	while i < layer.features.len() {
		let feature = &layer.features[i];


		if previous_cwy != &feature.attributes.CWY{
			// the into() function on the next line is doing magic that i dont quite understand. Maybe its better than my previous solution which was .clone() ?
			match map_from_road_number.entry(previous_road.into()){ 
				Entry::Vacant(e)=>{
					e.insert(RoadDataByCwy::new_from_cwy(previous_cwy, (current_slice_start, i)));
				}
				Entry::Occupied(mut e)=>{
					e.insert(e.get().with_updated_cwy(previous_cwy, (current_slice_start, i)));
				}
			}
			current_slice_start = i;
		}

		if previous_road != &feature.attributes.ROAD {
			let new_first_letter = match feature.attributes.ROAD.chars().next() {
				Some(fl) => fl,
				None => ' ',
			};
			if new_first_letter != first_letter{
				first_letter = new_first_letter;
				map_from_road_number = map_from_first_letter.entry(first_letter).or_default();
			}
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

	Ok(map_from_first_letter)
}
