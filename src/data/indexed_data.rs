use std::{path::Path, fs::File, error::Error};
use lz_fear;
use reqwest;
use serde_json;

use crate::{helpers::ErrorWithStaticMessage, filters::query_parameters::RequestedCwy};

use super::{
    esri_serde::{LayerDownloadChunk, LayerSaved, LayerSavedFeature},
    index::{index_data, LookupMap, RoadDataByCwy}
};


pub struct IndexedData {
    data:LayerSaved,
    index:LookupMap
}

impl IndexedData {

    /// Load existing data from the cache file path, or try to download data 
    /// and save it to the cache file path
    pub async fn load(
        path_to_data_cache_file:&String,
        url_to_download_new_data:&String,
        force_update:&bool,
    ) -> Result<Self, Box<dyn Error>>{

        let data = Self::read_or_update_cache_data(path_to_data_cache_file,url_to_download_new_data,force_update).await?;
        println!("INFO: Indexing data");
        let index = index_data(&data)?;
        return Ok(Self{
            data,
            index
        })
    }

    fn get_road_by_cwy(&self, road_name:&String) -> Result<&RoadDataByCwy, ErrorWithStaticMessage> {
        
        // try to get the first letter of the road name
        let first_letter = road_name.chars().next().ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. First letter of 'road' did not match any in lookup table.")
        )?;
        
        // lookup the list of roads that start with that letter
        let roads_with_first_letter = self.index.get(&first_letter).ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. Could not get first letter of road.")
        )?;

        // find the matching road
        roads_with_first_letter.get(road_name).ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. 'road' not found in second level lookup table.")
        )
    }

    pub fn query(&self, road_name:&String, cwy:&RequestedCwy) -> Result<impl Iterator<Item = &LayerSavedFeature>, ErrorWithStaticMessage> {
        let road_data_by_cwy = self.get_road_by_cwy(road_name)?;
        let feature_iterator = cwy
            .into_iter()
            .filter_map(|cwy|{
                if let Some(indexes) = road_data_by_cwy[&cwy]{
                    Some(&self.data.features[indexes.0..indexes.1])
                }else{
                    None
                }
            })
            .flatten();
        Ok(feature_iterator)
    }

    async fn read_or_update_cache_data (
        path_to_data_cache_file:&String,
        url_to_download_new_data:&String,
        force_update:&bool,
    ) -> Result<LayerSaved, Box<dyn Error>> {
        
        let path_to_data_cache_file = Path::new(path_to_data_cache_file);
    
        if !force_update {
            println!("INFO: Loading data from '{}'.", path_to_data_cache_file.display());
            match Self::load_data_from_file(path_to_data_cache_file) {
                Ok(loaded_data) => return Ok(loaded_data),
                Err(error_message) => println!("WARNING: Could not open the specified data file because '{}'. Will try to download fresh data.", error_message),
            }
        }
    
        if let Some(parent) = path_to_data_cache_file.parent() {
            if !parent.is_dir() {
                println!("WARNING: The directory '{}' does not exist. Creating it now", parent.display());
                match std::fs::create_dir_all(parent) {
                    Ok(_) => (),
                    Err(error_message) => {
                        println!("WARNING: Could not create the directory '{}' because '{}'", parent.display(), error_message);
                    }
                }
            }
        }
    
        if path_to_data_cache_file.is_file() {
            println!("WARNING: The file '{}' already exists. Attempting to delete it", path_to_data_cache_file.display());
            match std::fs::remove_file(path_to_data_cache_file) {
                Ok(_) => (),
                Err(error_message) => {
                    println!("WARNING: Could not delete the file '{}' because '{}'", path_to_data_cache_file.display(), error_message);
                }
            };
        }
        
        
        println!("INFO: Downloading fresh data");
        println!("INFO: Using '{}'", url_to_download_new_data);
        println!("INFO: Please wait, this can take some time...");
        let new_data = Self::download_data(url_to_download_new_data).await?;
        
        println!("INFO: Saving data to file.");
        match Self::save_data_to_file(path_to_data_cache_file, &new_data){
            Ok(_) => (),
            Err(error_message) => {
                println!("WARNING: Could not save the specified data file '{}' because '{}'", path_to_data_cache_file.display(), error_message);
                println!("WARNING: Data will be downloaded again next time the program is run");
            }
        }
        
    
        Ok(new_data)
    }

    async fn download_data(
        url:&String,
    ) -> Result<LayerSaved, Box<dyn Error>> {
        
        let mut document_to_save = LayerSaved {
            features: Vec::with_capacity(183_000),
        };

        let mut offset: usize = 0;
        const MAX_LOOPS: usize = 500;
        let mut loop_safety_limit = MAX_LOOPS;
        loop {
            loop_safety_limit -= 1;
            if loop_safety_limit <= 0 {
                return Err(
                    //Box::new(ErrorWithStaticMessage::new(format!("Download failed. Loop safety limit of '{}' exceeded.", MAX_LOOPS).as_str()))
                    Box::new(ErrorWithStaticMessage::new("Download failed. Loop safety limit exceeded."))
                );
            }

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
        
        println!("INFO: Download completed. Sorting data.");
        document_to_save
            .features
            .sort_by(|a, b| a.attributes.cmp(&b.attributes));
        Ok(document_to_save)
    }

    fn load_data_from_file(file_path:&Path) -> Result<LayerSaved, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let lz_frame_reader = lz_fear::framed::LZ4FrameReader::new(file)?;
        let lz_frame_io_reader = lz_frame_reader.into_read();
        let result = serde_json::from_reader(lz_frame_io_reader)?;
        Ok(result)
    }

    fn save_data_to_file(file_path:&Path, data:&LayerSaved) -> Result<(), Box<dyn Error>> {
        let file_out = File::create(file_path)?;
        let res = serde_json::to_vec(&data)?;
        let compressor = lz_fear::framed::CompressionSettings::default();
        let result = compressor.compress(&res[..], &file_out)?;
        Ok(result)
    }

}