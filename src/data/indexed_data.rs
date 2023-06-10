use std::{path::Path, fs::File, error::Error};
use lz_fear;
use reqwest;
use serde_json;

use crate::{helpers::ErrorWithStaticMessage, routes::query_parameters::RequestedCwy};

use super::{
    esri_serde::{LayerDownloadChunk, LayerSaved, LayerSavedFeature},
    index::{index_data, LookupMap, RoadDataByCwy}
};


pub struct IndexedData {
    data:LayerSaved,
    index:LookupMap
}

impl IndexedData {

    pub async fn load(
        path_to_data_cache_file:&String,
        url_to_download_new_data:&String,
        force_update:&bool,
    ) -> Result<Self, Box<dyn Error>>{

        let data = Self::read_or_update_cache_data(path_to_data_cache_file,url_to_download_new_data,force_update).await?;
        println!("Indexing data");
        let index = index_data(&data)?;
        return Ok(Self{
            data,
            index
        })
    }

    fn get_road(&self, road_name:&String) -> Result<&RoadDataByCwy, ErrorWithStaticMessage> {
        
        // try to get the first letter of the road name
        let first_letter = road_name.chars().next().ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. First letter of road did not match any in lookup table.")
        )?;
        
        // lookup the list of roads that start with that letter
        let roads_with_first_letter = self.index.get(&first_letter).ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. Could not get first letter of road.")
        )?;

        // find the matching road
        roads_with_first_letter.get(road_name).ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. Name not found in second level lookup table.")
        )
    }

    pub fn query(&self, road_name:&String, cwy:&RequestedCwy) -> Result<impl Iterator<Item = &LayerSavedFeature>, ErrorWithStaticMessage> {
        let road_data_by_cwy = self.get_road(road_name)?;
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
            match Self::load_data_from_file(path_to_data_cache_file) {
                Ok(response) => return Ok(response),
                Err(error_message) => println!("WARNING:Error while loading data: {}", error_message),
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
        let new_data = Self::download_data(url_to_download_new_data).await?;
        
        println!("Saving data");
        let file_out = File::create(path_to_data_cache_file)?;
        let res = serde_json::to_vec(&new_data)?;
        let compressor = lz_fear::framed::CompressionSettings::default();
        compressor.compress(&res[..], &file_out)?;
    
        Ok(new_data)
    }

    async fn download_data(
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

    fn load_data_from_file(s:&Path) -> Result<LayerSaved, Box<dyn std::error::Error>> {
        println!("Loading data from file.");
        let file = File::open(s)?;
        let lz_frame_reader = lz_fear::framed::LZ4FrameReader::new(file)?;
        let lz_frame_io_reader = lz_frame_reader.into_read();
        let result: LayerSaved = serde_json::from_reader(lz_frame_io_reader)?;
        Ok(result)
    }
}